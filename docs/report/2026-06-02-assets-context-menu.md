# Implementação do Menu de Contexto para Assets

**Data:** 02 de Junho de 2026  
**Objetivo:** Adicionar recursos de Qualidade de Vida (QoL) no front-end do Mundam introduzindo um menu de contexto para interação avançada com arquivos de mídia (Assets) em diferentes tipos de layouts e listagens.

## Resumo das Modificações

O trabalho consistiu em criar um componente de Menu de Contexto reaproveitável, injetá-lo na infraestrutura dos diferentes layouts de exibição e configurar o back-end em Rust (Tauri) para autorizar nativamente o acesso ao sistema de arquivos para tais ações. 

### 1. Criação do Componente `AssetContextMenu`
**Arquivo:** `src/components/features/viewport/components/AssetContextMenu.tsx`

Seguindo o padrão de design adotado no sistema e inspirado no `FolderContextMenu`, criamos o `AssetContextMenu`. Foram implementadas três ações principais, cada uma disparando funcionalidades chaves usando plugins do ecossistema Tauri e APIs da Web:
- **"Open file":** Abre o arquivo no seu editor de imagem/vídeo padrão do sistema operacional, utilizando a função `openPath` do pacote `@tauri-apps/plugin-opener`. 
- **"Reveal in OS":** Abre o gerenciador de arquivos (Finder no macOS, Explorer no Windows) focando diretamente no arquivo correspondente, por meio do `revealItemInDir`.
- **"Copy Path":** Copia o caminho absoluto do arquivo para a área de transferência do sistema do usuário, usando a API nativa `navigator.clipboard.writeText`.

### 2. Integração do Menu nos Layouts

A arquitetura do Mundam utiliza componentes de listagem virtualizados altamente performáticos que precisaram ser adaptados para interceptar interações com o botão direito do mouse:

- **Layouts Grid e Masonry:**
  - **Arquivos:** `VirtualGridView.tsx` e `VirtualMasonry.tsx`
  - Foi criado e propagado um evento `handleContextMenu` para interceptar o `onContextMenu` lançado pelo `AssetCard`. Isso nos permitiu coletar o ID do arquivo sem afetar a navegação primária e registrar a posição do mouse na tela.
  
- **Layout List (Table):**
  - **Arquivos:** `VirtualListView.tsx`, `Table.tsx`, `TableRow.tsx` e `types.ts`
  - O Table precisou de refatorações de infraestrutura para que um evento de botão direito passasse a ser validado até o topo do Virtual DOM da tabela.
  - Atualizamos os `TableProps` introduzindo o callback `onRowContextMenu`.
  - Lidamos com uma falha local atualizando a configuração do `splitProps` no `Table.tsx`, garantindo que a nova propriedade fosse exposta corretamente no objeto `local`.
  - Com isso o `VirtualListView.tsx` passou a reagir aos clicks-direitos exatamente igual as opções de Grid e Masonry.

### 3. Ajuste de Permissões e Segurança no Tauri
**Arquivo:** `src-tauri/capabilities/default.json`

A V2 do framework Tauri lida com a segurança do plugin `opener` não assumindo que todos os caminhos do disco podem ser arbitrariamente executados. 
- Adicionamos a capacidade `"opener:allow-open-path"` definindo regras de `path: "**"` autorizando que imagens sejam abertas via Mundam.
- Adicionamos `"opener:allow-reveal-item-in-dir"` para possibilitar que a função Revelar no SO abra as pastas correspondentes.

---

## Possíveis Melhorias e Futuros Recursos Agregáveis

A adição desse menu de contexto fornece a fundação perfeita para acoplar ainda mais utilitários sem prejudicar o visual minimalista do Mundam. Aqui estão sugestões e outros recursos que podem agregar grande valor e facilidade de uso:

### Melhorias no Menu de Contexto de Assets
1. **Copiar Imagem Real para Área de Transferência:** Em vez de copiar apenas o caminho (Path), uma opção para copiar o arquivo diretamente para o clipboard do sistema (para colar em um chat, Photoshop ou Figma, por exemplo).
2. **Favoritar (Star/Heart):** Marcar o arquivo rapidamente como favorito pela lista.
3. **Conversão de Formato Embutida:** Um pequeno submenu permitindo conversão rápida ("Converter para PNG/JPEG").
4. **Adicionar/Remover de um Smart Folder ou Álbum:** Fluxos para organizar arquivos com facilidade nas pastas virtuais diretamente pelo menu.
5. **Quick Look Nativo (Preview):** Acionar um preview flutuante ou full-screen na própria UI, sem precisar abrir um programa externo (Semelhante à barra de espaço no macOS).

### Operações em Lote e Sistema de Arquivos
1. **Contexto de Seleção Múltipla:** Se o usuário possui 10 imagens selecionadas e clica com o botão direito, o Menu poderia exibir operações em lote (Ex: "Atribuir Tags a 10 arquivos" ou "Deletar 10 arquivos").
2. **Operações Críticas no Disco (Rename / Delete):** Possibilitar excluir do disco (`Move to Trash`) ou Renomear Arquivo diretamente pelo menu de contexto na listagem do Mundam, refletindo isso pelo Filesystem Watcher do back-end.
3. **Compartilhar (Share menu):** Invocar a janela nativa de compartilhamento do SO (AirDrop no macOs) selecionando uma foto a partir do próprio aplicativo.
