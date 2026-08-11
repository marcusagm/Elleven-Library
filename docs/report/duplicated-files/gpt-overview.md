A melhor abordagem, no estado atual do Mundam, é tratar duplicidade como um **subproduto do pipeline de indexação**, e não como uma busca ad hoc disparada pela UI. Isso encaixa bem na arquitetura que você já tem: o app é local-first, usa indexação incremental, geração assíncrona de thumbnails, SQLite/SQLx e já se propõe a ter detecção de duplicados por hash no roadmap. Também já existe uma base forte de galeria virtualizada, inspeção em tela cheia e filtros/smart folders, o que ajuda muito na apresentação dos resultados.

## 1) Como eu estruturaria a detecção

Eu separaria em três níveis:

**Nível A — duplicados exatos**
Mesmo conteúdo binário, ou conteúdo equivalente com variação irrelevante para o usuário. Aqui entram arquivos idênticos, reexportados sem alteração, cópias com nome diferente, etc. A regra principal deve ser um hash rápido e estável do arquivo original, guardado no banco como fingerprint primária.

**Nível B — duplicados visuais próximos**
Imagens redimensionadas, recomprimidas, com pequenas correções de cor, rotação ou perda de qualidade. Aqui vale usar um *perceptual fingerprint* por imagem renderizada/thumbnail, para agrupar candidatos parecidos.

**Nível C — duplicados derivados**
Crops, recortes, versões com texto sobreposto, edições parciais. Esses casos não devem depender só de hash perceptual simples; precisam de uma etapa de verificação mais cara, comparando regiões, proporção de área comum e score de correspondência. A decisão final precisa ser configurável pelo usuário, porque “duplicado” pode significar coisas diferentes conforme o fluxo de trabalho.

## 2) Onde isso entra na arquitetura atual

A arquitetura atual já tem exatamente os pontos certos para isso: o backend em Rust roda com **indexer/watcher**, **thumbnail worker**, **SQLite** e **protocolos próprios**, tudo orquestrado no `src-tauri/src/lib.rs`. O backend sobe os watchers dos roots, inicializa o banco, cria o worker de thumbnails e expõe comandos Tauri para a UI.

O lugar mais limpo para a deduplicação é:

* **Indexer**: detecta novo arquivo, mudança, rename e delete.
* **Fingerprint service**: calcula e persiste hashes.
* **Duplicate matcher**: agrupa candidatos e mantém o estado dos grupos.
* **Asset Ledger / Command layer**: executa ações mutacionais do usuário com atomicidade e audit trail.

Isso combina com o documento do `Asset Ledger`, que já define o ledger como o guardião das mutações, validação de estado, transações atômicas e log de auditoria. Então, remover, marcar como ignorado, mesclar ou manter um asset precisa passar por esse núcleo, não por lógica solta na UI.

## 3) Modelo de dados que eu criaria

Eu criaria uma camada nova, algo como `duplicate/` ou `fingerprint/`, com estas entidades principais:

* `asset_fingerprint`

  * `asset_id`
  * `content_hash`
  * `perceptual_hash`
  * `thumbnail_hash`
  * `width`, `height`
  * `mime`, `format_family`
  * `color_profile`, `orientation`
  * `indexed_at`, `version`

* `duplicate_group`

  * `id`
  * `rule_set_id`
  * `group_type` (`exact`, `near`, `derived`)
  * `canonical_asset_id`
  * `confidence`
  * `status` (`open`, `reviewed`, `ignored`, `resolved`)

* `duplicate_candidate`

  * `group_id`
  * `asset_id`
  * `score`
  * `reasons` (`same_hash`, `same_phash_bucket`, `crop_overlap`, `same_dimensions`, etc.)

* `duplicate_resolution`

  * registra a decisão do usuário, para não reaparecerem os mesmos grupos depois.

## 4) Como achar imagens cortadas ou modificadas

Para o caso que você citou — imagens cortadas ou modificadas ainda consideradas duplicadas — eu usaria uma estratégia em camadas:

**1. Pré-filtro barato**
Agrupar por família, dimensão aproximada, faixa de tamanho, orientação, histograma de cor ou hash perceptual. Isso reduz brutalmente o custo.

**2. Similaridade visual**
Comparar miniaturas normalizadas em múltiplas escalas. Para crop/resize/recompressão, um único pHash costuma falhar; um conjunto de features por blocos funciona melhor.

**3. Verificação semântica de corte**
Se a imagem A parece ser um recorte de B, testar cobertura parcial: quanto da área de A “cabe” dentro de B após normalização. Aqui o score pode ser do tipo:

* 1.0 = idêntico
* 0.85+ = fortemente derivado
* 0.65–0.85 = parecido, revisar manualmente
* abaixo disso = só similar, não duplicado

**4. Regras configuráveis pelo usuário**
Exemplos:

* considerar crop como duplicado
* ignorar diferenças de resolução
* exigir mesmo conteúdo visual + mesmo aspect ratio
* aceitar mudanças de cor/contraste
* agrupar versões de um mesmo PSD/exportado em PNG/JPG

Isso é importante porque, para artistas, uma versão cortada pode ser “duplicata operacional”, enquanto para outra pessoa pode ser um asset legítimo diferente.

## 5) Pipeline ideal de execução

Eu não faria essa checagem no thread da UI nem em toda listagem. O fluxo ideal seria:

1. O watcher/indexer detecta o arquivo.
2. O indexador cria ou atualiza o asset no SQLite.
3. Enfileira jobs:

   * thumbnail
   * fingerprint
   * duplicate candidate scan
4. O matcher consulta candidatos já conhecidos.
5. Se surgir grupo novo, emite evento para a UI.
6. A UI mostra o grupo na tela de duplicados.
7. O usuário resolve.
8. O Asset Ledger aplica a decisão e grava histórico.

Como o Mundam já usa processamento paralelo em background para thumbnails e indexação incremental, essa arquitetura mantém a UI leve e evita travar a navegação da galeria.

## 6) Interface ideal para revisar duplicados

Eu faria isso como uma **área própria de triagem**, não como popup simples.

### Estrutura da tela

* **Coluna lateral**: filtros e regras.
* **Painel principal**: grupos de duplicados.
* **Painel inferior ou lateral direito**: comparação detalhada.
* **Topo**: resumo geral, contadores e ações em lote.

### Como apresentar cada grupo

Cada grupo deve mostrar:

* miniatura do candidato “canônico”
* miniaturas dos demais assets
* score de similaridade
* caminho completo
* tamanho, resolução, data, formato
* indicação visual do tipo de duplicidade:

  * exato
  * provável
  * derivado
  * revisão manual

### Interação recomendada

* Clique em um grupo abre modo comparação.
* Permitir seleção de “manter”.
* Botões rápidos:

  * **Manter este**
  * **Manter o maior**
  * **Manter o mais antigo**
  * **Excluir selecionados**
  * **Ignorar grupo**
  * **Mesclar metadados/tags**
  * **Abrir pasta**
* Atalhos de teclado para triagem em massa.

## 7) UX de comparação que vale muito a pena

Para duplicados visuais, o ideal é um comparador em **split view sincronizado**:

* zoom travado entre imagens
* pan sincronizado
* overlay alternável
* diferença de dimensões destacada
* histograma/EXIF lado a lado
* indicação de crop detectado

Para artista, isso é muito melhor do que só uma lista textual, porque a decisão é visual.

## 8) Fluxo de decisão do usuário

Eu desenharia a resolução assim:

**A. Revisão rápida**

* mostrar apenas grupos acima de um score alto
* ação de um clique para os casos óbvios

**B. Revisão detalhada**

* para casos ambíguos
* compara 2 a 4 candidatos
* mostra razões do agrupamento

**C. Persistência da decisão**

* salvar a regra aplicada ao grupo
* marcar assets ignorados
* permitir “aplicar mesma decisão em similares futuros”

Isso evita retrabalho e dá sensação de sistema inteligente de verdade.

## 9) Minha recomendação objetiva

Se eu tivesse que escolher uma direção única, seria esta:

**Criar um serviço backend de fingerprints + matcher incremental, com UI de revisão baseada em grupos de duplicados e comparação visual.**
Não tentaria resolver tudo só com hash de arquivo. Para o Mundam, o valor está em reconhecer:

* duplicado exato,
* versão reexportada,
* crop/modificação leve,
* e permitir que o usuário defina o que conta como duplicata.

Se você quiser, eu posso transformar isso em uma proposta técnica mais concreta com módulos Rust, tabelas SQLite e eventos Tauri já nomeados para encaixar direto no código do Mundam.
