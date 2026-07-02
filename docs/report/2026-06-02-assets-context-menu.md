# Implementação do Menu de Contexto para Assets

**Data:** 02 de Junho de 2026  
**Objetivo:** Adicionar recursos de Qualidade de Vida (QoL) no front-end do Mundam introduzindo um menu de contexto para interação avançada com arquivos de mídia (Assets) em diferentes tipos de layouts e listagens.

## Resumo das Modificações

O trabalho consistiu em criar um componente de Menu de Contexto reaproveitável, injetá-lo na infraestrutura dos diferentes layouts de exibição e configurar o back-end em Rust (Tauri) para autorizar nativamente o acesso ao sistema de arquivos para tais ações. 

### 1. Criação e Evolução do Componente `AssetContextMenu`
**Arquivo:** `src/components/features/viewport/components/AssetContextMenu.tsx`

Seguindo o padrão de design adotado no sistema e inspirado no `FolderContextMenu`, criamos e evoluímos o `AssetContextMenu`. Foram implementadas ações principais e dinâmicas que se adaptam se múltiplos itens estiverem selecionados (utilizando o `selectionStore`):
- **"Open file(s)":** Abre o(s) arquivo(s) no seu editor padrão do sistema operacional, utilizando a função `openPath` do pacote `@tauri-apps/plugin-opener`. 
- **"Reveal in OS":** Abre o gerenciador de arquivos (Finder no macOS, Explorer no Windows) focando diretamente no arquivo correspondente. Esta opção é inteligentemente **omitida** se mais de um arquivo estiver selecionado, evintando a abertura caótica de múltiplas janelas do SO.
- **"Copy Path(s)":** Copia o caminho absoluto do arquivo para a área de transferência. Em caso de múltipla seleção, os caminhos são copiados em formato de lista (quebras de linha), usando a API nativa `navigator.clipboard.writeText`.
- **"Copy File(s)":** Diferente de copiar o caminho (texto), esta funcionalidade possibilita copiar o arquivo físico binário (imagem, vídeo, áudio, etc) para a área de transferência do Sistema Operacional (ex: para colar num software de edição, chat ou Finder).
- **"Rename":** Invoca o componente `PromptModal` para permitir a renomeação segura de um arquivo. Ao confirmar, o arquivo físico no disco é renomeado usando `@tauri-apps/plugin-fs`. Devido à arquitetura robusta do nosso Indexer (e seu `Heuristic Matcher`), o backend de Rust detecta automaticamente o novo nome e sincroniza isso no banco de dados através da instrução `LedgerCommand::UpdateAsset`, garantindo que não existam conflitos e eliminando necessidade de complexidade dupla na chamada API. Essa opção só é exibida se exatamente UM item for selecionado.

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
**Arquivos:** `src-tauri/capabilities/default.json` e `src-tauri/src/delivery/tauri/commands/mutations.rs`

A V2 do framework Tauri lida com a segurança de plugins não assumindo que todos os caminhos do disco podem ser arbitrariamente executados.
- Instalamos o novo plugin `@tauri-apps/plugin-clipboard-manager`.
- Adicionamos a capacidade `"opener:allow-open-path"` e `"opener:allow-reveal-item-in-dir"` configuradas com `path: "**"` no `default.json` para garantir que as mídias possam ser reveladas/abertas.
- Também concedemos as permissões para nossas chamadas RPC dedicadas: `"allow-copy-files-to-clipboard"` e `"allow-rename-file"`.
- **Implementação de Renomeação em Rust:** Ao invés de dependermos do `fs.rename` do front-end — que nos forçaria a diminuir a segurança global da aplicação ao requerer acesso de gravação indiscriminado (`fs:write-all`) — criamos o comando `rename_file` no Rust, o que nos garante agilidade e controle absolutos e isolados sobre as movimentações físicas, sem ferir a postura de segurança da UI.
- **Implementação Híbrida de Clipboard para Arquivos Reais (JXA):** Já que o plugin padrão de clipboard nativo da web e o do Tauri possuem severas limitações para lidar com arquivos binários diversos em lote de forma agnóstica na área de transferência (NSPasteboard/arboard), construímos um comando RPC robusto e customizado em Rust (`copy_files_to_clipboard`). Para o macOS, o comando invoca debaixo dos panos o JXA (JavaScript for Automation) que faz a ponte com a API em `Objective-C` (AppKit), injetando perfeitamente instâncias de `NSURL` (e convertendo para `«class furl»` ou `NSFilenamesPboardType`) nativamente na Pasteboard, permitindo ao usuário colar os arquivos perfeitamente no *Finder* ou na *Área de Trabalho* com a experiência de OS pura.

---

## Possíveis Melhorias e Futuros Recursos Agregáveis

A adição desse menu de contexto fornece a fundação perfeita para acoplar ainda mais utilitários sem prejudicar o visual minimalista do Mundam. Aqui estão sugestões e outros recursos que podem agregar grande valor e facilidade de uso:

### Melhorias no Menu de Contexto de Assets
1. **Favoritar (Star/Heart):** Marcar o arquivo rapidamente como favorito pela lista.
2. **Conversão de Formato Embutida:** Um pequeno submenu permitindo conversão rápida ("Converter para PNG/JPEG/WEBP").
3. **Adicionar/Remover de um Smart Folder ou Álbum:** Fluxos para organizar arquivos com facilidade nas pastas virtuais diretamente pelo menu.
4. **Quick Look Nativo (Preview):** Acionar um preview flutuante ou full-screen na própria UI, sem precisar abrir um programa externo (Semelhante à barra de espaço no macOS).

### Operações em Lote e Sistema de Arquivos
1. **Suporte de Cópia Binária (Clipboard) no Windows/Linux:** Expandir o comando atual em Rust `copy_files_to_clipboard` para invocar os fluxos adequados da área de transferência nos Sistemas Operacionais Windows e Linux, mantendo a paridade de features que atualmente está focada de forma profunda no macOS.
2. **Atribuir/Remover Tags Múltiplas:** Possibilidade de atribuir ou remover tags à todos os assets selecionados de uma vez através do menu de contexto.
3. **Exclusão de arquivos críticos:** Possibilitar excluir do disco (`Move to Trash`) diretamente pelo menu de contexto na listagem do Mundam, o Indexer sincronizaria facilmente isso com o DB (semelhante ao que faz para renomear).
4. **Compartilhar (Share menu):** Invocar a janela nativa de compartilhamento do SO (AirDrop no macOs) selecionando uma foto a partir do próprio aplicativo.
