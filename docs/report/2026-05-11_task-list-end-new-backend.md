# Recursos

## Atalhos

- [ ] Adicionar atalho para omitir as sidebars com a tecla "tab" como padrão.

## Acessibilidade

- [ ] Adicionar verificações de `prefers-reduced-motion` no CSS para animações de layout.

## Home

- [ ] Mostrar a lista de arquivos mais recentes de todas as pastas indexadas.
- [ ] Adicionar contator de pastas indexadas e arquivos na capa da aplicação.
- [ ] Adicionar contator de arquivos por formato, um contador por tipo como imagem, audio, video, documentos, etc.
- [ ] Adicionar contator de tags.
- [ ] Adicioanr contator de smart folders.
- [ ] Adicionar contator de arquivos duplicados.
- [ ] Adicionar contator de arquivos favoritos.
- [ ] Adicionar contator de arquivos na lixeira.

## Library

- [ ] Adicionar lixeira
- [ ] Adicionar arquivos favoritos

## Item Inspector

- [ ] Copiar cor extraida no formato rgba, rgb, hex, hsl, hsv, hwb, cmyk.
- [ ] Adicionar atalho para buscar por cores diretamente pela paleta de cores selecionada.
- [ ] Persistência de paineis abertos ou colapsados, por exemplo se o painel general info for aberto, deve permacer aberto até o usuário fecha-lo.

## Seacrh

- [ ] Adicionar botão de limpar texto de busca.
- [ ] Verificar fuzzy search, pois parece não estar funcionando bem.
- [ ] Adicionar sistema de sugestões.
- [ ] Permitir a busca por Harmonia de cores 
- [ ] Busca recursiva por tags, incluir tags filhas na busca.
- [ ] Busca recursiva por pastas, incluir pastas filhas na busca.

## Folders

- [ ] Criar nova subpasta
- [ ] Renomear pasta
- [ ] Mover pasta
- [ ] Excluir pasta

## Listagem

- [ ] Adicionar opção de copiar caminho do arquivo.
- [ ] Adicionar visualização por icones.
- [ ] Implementar a funcionalidade de clicar e arrastar no vazio para selecionar múltiplos itens na Grid e no mansory.

## Assets

- [ ] Abrir arquivo no editor padrão do sistema operacional.
- [ ] Abrir pasta onde se encontra o arquivo.
- [ ] Adicionar asset como favorito.
- [ ] Converter arquivo para outros formatos.
- [ ] Verificar duplicidade de arquivos.
- [ ] Adicionar asset arrastando para a janela do Mundam.
- [ ] Adicionar recurso de virar imagem automaticamente de acordo com o metadado EXIF orientation.

## Visualizador de assets

### Geral

- [ ] Adicionar nome do arquivo acima da toolbar.
- [ ] Ao carregar o arquivo, mostrar um loader de forma que não trave a interface do usuário, permitindo que ele possa sair do itemview quando quiser, cancelando o processo se necessário.
- [ ] Permitir acesso a detalhes do arquivo na item view.

### Imagens

- [ ] Adicionar opção de copiar imagem na visualização de imagens.
- [ ] Adicionar sistema de notas para imagens permitindo texto e desenhos.
- [ ] Visualizardor de gifs com controle de play/pause, de velocidade de reprodução e timeline para navigating entre frames.

### Áudio

- [ ] No lugar onde está atualmente um icone estático, mostrar Espectro de Frequência.
- [ ] Adicionar sistema de notas atribuidas a um tempo determinado no áudio.

### Vídeos

- [ ] Adicionar sistema de notas atribuidas a um tempo determinado no video.

# Bugs

- [x] Filtro de por itens com tags não está funcionando
- [x] Restaurar o menu de contexto da pasta raiz.
- [x] Melhorar a atualização de dados de arquivos para evitar flicker e interface.
      **Solução:** O bridge de eventos em `lib.rs` emitia `library:batch-change` com `needs_refresh: true` para **todos** os `DomainEvent`, incluindo `AssetMetadataUpdated`, `AssetStateChanged` e `AssetTagsUpdated`. Cada um desses disparava `libraryActions.refreshAssets()` que reconstruía toda a lista com `reconcile()`. Durante indexação, centenas de `AssetCreated` causavam centenas de full-refreshes.
      **Fix:** Removidos `AssetMetadataUpdated`, `AssetStateChanged` e `AssetTagsUpdated` do match no bridge — esses eventos já são tratados granularmente por `thumbnail:ready` e `extraction:completed`. Apenas eventos estruturais (`AssetCreated`, `FsPathDeleted`, `FsPathRenamed`, `AssetFolderChanged`, `FolderMetadataUpdated`) disparam `batch-change`. Debounce do `handleBatchChange` aumentado de 500ms para 1500ms.
      **Arquivos:** `src-tauri/src/lib.rs`, `src/core/store/library/libraryActions.ts`
- [x] Corrigir flicker e interface ao atualizar a thumbnail dos arquivos.
      **Solução:** Quando o `ThumbnailWorker` completava uma thumbnail, o `LedgerCommand::UpdateThumbnail` emitia `DomainEvent::ThumbnailGenerated` que causava DOIS efeitos no bridge: (1) `thumbnail:ready` com atualização cirúrgica ✅ e (2) `library:batch-change` via match de `AssetStateChanged` → full refresh ❌. O segundo efeito causava o flicker: a thumbnail aparecia via `thumbnail:ready`, depois a UI inteira piscava durante o `refreshAssets()`.
      **Fix:** A remoção de `AssetStateChanged` e `AssetMetadataUpdated` do bridge (mesma mudança do item anterior) elimina o double-render. Agora apenas o evento `thumbnail:ready` atualiza a UI, de forma cirúrgica e sem flicker.
      **Arquivos:** `src-tauri/src/lib.rs`
- [x] Encontrar forma de melhorar a experiência ao exibir thumbnails, mostrando ícones para arquivos quebrados, ou que não possuem geradores de thumbnail 
      **Solução:** Criado componente `FileIcon.tsx` que renderiza o SVG do Mundam inline com cores dinâmicas baseadas no `mediaType` do asset (Image → teal, Video → coral, Audio → purple, Project → orange, etc.) e texto da extensão do arquivo. O `Thumbnail.tsx` foi reescrito para usar três estados visuais distintos baseados no campo `state` do asset:
      - **Loader (spinner):** Quando o asset ainda está na fila de processamento (`state` = Discovered, Probing, ou Indexed) e não tem thumbnail — indica que o arquivo será processado em breve.
      - **FileIcon (ícone SVG):** Quando o asset já foi processado (`state` = Thumbnailed, Idle, Unknown) mas não possui thumbnail — indica que o formato não tem suporte de extração ou ocorreu um erro permanente.
      - **Thumbnail (imagem):** Quando o `thumbnail_path` existe e a imagem carregou com sucesso.
      O campo `state` foi adicionado ao tipo `AssetItem` no frontend (o backend já serializava, mas era ignorado).
      **Arquivos:** `src/components/features/viewport/assets/FileIcon.tsx` (novo), `src/components/features/viewport/assets/file-icon.css` (novo), `src/components/features/viewport/assets/Thumbnail.tsx`, `src/components/features/viewport/assets/AssetCard.tsx`, `src/types/index.ts`
- [x] Remover MCP do tauri já que não esta sendo usado devido a vários problemas.
      **Solução:** Removida a dependência `tauri-plugin-mcp-bridge = "0.2"` do `Cargo.toml` e a linha `.plugin(tauri_plugin_mcp_bridge::init())` do `lib.rs`. Não existiam referências no frontend nem em configurações JSON.
      **Arquivos:** `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`
- [x] Thumbnail worker está falhando ao gerar thumbnails para alguns arquivos, e entrando em um loop, coisa que não acontecia na arquitetura v1.
      **Solução:** A query `get_assets_needing_thumbnails` em `queries.rs` buscava `WHERE thumbnail_path IS NULL`. Quando um formato sem suporte era processado com erro, o `thumbnail_path` permanecia NULL mas o `state` era atualizado para `Thumbnailed`. O worker buscava os mesmos IDs infinitamente. Fix: adicionado `AND state != 'Thumbnailed'` na query SQL, excluindo assets já processados (independente de sucesso ou falha) da fila de trabalho.
      **Arquivos:** `src-tauri/src/infra/database/queries.rs`
- [ ] A geração de thumbnails deve sempre priorizar os assets que estão visiveis na tela. Atualmente na V2 a prioridade de geração de thumbnails parece não funcionar.
- [ ] A extração de informação de cores tambem deve priorizar os assets visiveis ou selecionados.
- [ ] Ao mover uma pasta e todo seu conteúdo para outra pasta indexada, a hieraquia não foi refeita e a alteração do path não foi refletida corretamente no banco de dados.

# Códigos

## Core

- [ ] Melhorar a forma de resgitro de formatos em /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/registry.rs e mod.rs
- [ ] Testar todos os comandas e encontar uma forma melhor de organização /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/command.rs
- [ ] Melhorar a organização dos arquivos e pastas em /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/models/*
- [ ] Verificar o funcionamento do core/repository e sua interação com o ledgerer
- [ ] Verificar o funcionamento do core/settings e sua interação com o ledgerer
- [ ] Analisar a pasta core/workflows e verificar se existe algo que podemos retirar ou organizar melhor.

## Infra

- [ ] Analisar /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/infra/database/ para dividir o arquivo e organizar melhor. Muitos arquivos com muitas linhas de código.

## Processing

- [ ] O ideal é que cada formato junto com seus alias, tenho um arquivo exclusivo em 
      "src-tauri/src/processing/media" assim como é feito para arquivos `affinity_format`,
      `ai_format`, `aseprite_format`, `pdf_format`, entre outros, e funções comuns fiquem 
      agrupadas em "helpers" e em "extractors" se forem especificas para determinado formato. 
      Isso permitirá o tratamento exclusivo por formato definindo extração e inclusive 
      fallbacks diferentes para cada formato de arquivo, isso tambem deixará claro como é 
      o registro de cada formato, sem precisar ficar procurando em arquivos genericos como 
      "cad_format", "image_format" ou "audio_format".
- [ ] Organização dos formatos e extratores de arquivos
- [ ] Organização dos formatos e extratores de audio
- [ ] Organização dos formatos e extratores de documentos
- [x] Organização dos formatos e extratores de imagens
- [x] Organização dos formatos e extratores de fontes
- [ ] Organização dos formatos e extratores de modelos 3D
- [x] Organização dos formatos e extratores de projetos
- [x] Organização dos formatos e extratores de vetores
- [ ] Organização dos formatos e extratores de video


## Outros arquivos para verificar

- [ ] /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lib.rs
- [ ] /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lifecycle.rs

# Pendências de diferenças entre v1 e v2

- [ ] Verificar operações atômicas do banco de dados durante a movimentação de pastas em larga escala.
- [ ] Verificar implementação da lógica de "adoção" de pastas antigas do V1 no scanner V2.

## Suporte a formatos de arquivos

### Image
| Manual check | Extensão | V1   | V2   | V1 Notes                                         | V2 Notes                                                  | Manual check notes                                                                                                                                                                                        |
| :----------- | :------- | :--- | :--- | :----------------------------------------------- | :-------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🟢🟢           | `3fr`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `arw`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `avif`   | 🟢    | 🟢    | Suporte moderno.                                 | Nativo via `ModernImageFormatProvider`.                   | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `avifs`  | 🟢    | 🟢    | Suporte nativo (Sequência).                      | Nativo via `ModernImageFormatProvider`.                   | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `bmp`    | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `cr2`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `cr3`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `crw`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `cur`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | EThumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                     |
| 🟢🟢           | `dng`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `erf`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `exr`    | 🟢    | 🟢    | Estável.                                         | Suporte completo via `ExrFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok. No geral v2 está superior a v1                                                                                                                      |
| 🟢🟢           | `fff`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `gif`    | 🟢    | 🟢    | Estável, incluindo frames de animação.           | Suporte a frames de animação via FFmpeg/ImageUtils.       | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `hdr`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `heic`   | 🟠    | 🟠    | Problemas intermitentes de visualização (FFmpeg) | Instabilidade no decodificador M3U8 local (Sprint 10.12). | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `heif`   | 🟠    | 🟠    | Problemas intermitentes de visualização (FFmpeg) | Instabilidade no decodificador M3U8 local (Sprint 10.12). | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| ⚠️            | `heifs`  | 🟢    | 🟢    | Suporte nativo (Sequência).                      | Nativo via `ModernImageFormatProvider`.                   | Não testado                                                                                                                                                                                               |
| 🟢🟢           | `ico`    | 🟢    | 🟢    | Estável.                                         | Suporte completo via `IconFormatProvider`.                | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `iiq`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `jfif`   | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `jpe`    | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `jpeg`   | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `jpg`    | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `mef`    | 🟢    | 🟢    | Suporte LibRaw (Mamiya).                         | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `mos`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok. Nos metadados aparentemente não conseguiu extrair gerando um campo `Unknow to this library, or manufactuer-specific`                                |
| 🟢🟢           | `mrw`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `nef`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `nrw`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `orf`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `pam`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `pbm`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `pef`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `pgm`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `png`    | 🟢    | 🟢    | Estável.                                         | Estável. Suporte nativo via `ImageFormatProvider`.        | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `pnm`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `ppm`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `raf`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `rw2`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `rwl`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `sr2`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `srf`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `srw`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `tga`    | 🟢    | 🟢    | Suporte legado estável.                          | Nativo via `ImageFormatProvider`.                         | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `tif`    | 🟢    | 🟢    | Alias estável.                                   | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `tiff`   | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ImageFormatProvider`.                 | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `webp`   | 🟢    | 🟢    | Suporte completo.                                | Nativo via `ModernImageFormatProvider`.                   | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🟢🟢           | `kdc`    | 🔴    | 🔴    | Formatos antigos sem jpeg embutido               | Removido da registry V2 por obsolescência.                | Ajustado suporte para formatos legados (DC120) via sips e decodificação total via LibRaw para formatos modernos (EasyShare) sem preview embutido.                                                         |
| 🟢🟢           | `gpr`    | 🟢    | 🟢    | Estável via LibRaw.                              | Nativo em Rust (FFI c/ GoPro SDK oficial).                | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok - Implementação 100% nativa usando bind (C FFI) para o SDK oficial da GoPro, suportando todos os SOs sem dependências de terceiros.                  |
| 🟢🟢           | `jxl`    | 🟠    | 🟢    | Apenas ícone genérico/stub                       | Nativo em Rust (jxl-oxide).                               | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok - Implementação 100% nativa em Rust usando o decodificador `jxl-oxide`, suportando HDR (tone-mapping), modos VarDCT e Modular, ISOBMFF e codestream. |
| 🟢🟢           | `icns`   | 🟠    | 🟢    | Apenas ícone genérico/stub                       | Nativo em Rust (crate icns).                              | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok - Implementação 100% nativa capaz de extrair todas as resoluções e metadados com thumbnail/preview corretos.                                         |
| 🟢🟢           | `dds`    | 🟢    | 🟢    | Estável.                                         | Nativo em Rust (ddsfile + image_dds).                     | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok - Implementação via ddsfile contorna limitações do image-rs decodificando compactações BCn com suporte total a mipmaps para geração limpa.           |
| 🟢🟢           | `x3f`    | 🟢    | 🟢    | Estável via LibRaw.                              | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Thumbnail Ok, Metadados Ok, Dimensões Ok, Preview Ok                                                                                                                                                      |
| 🔴            | `dcr`    | 🔴    | 🔴    | Formatos antigos sem jpeg embutido               | Removido da registry V2 por obsolescência.                | Corretamente removido suporte                                                                                                                                                                             |
| 🟢🟠           | `raw`    | 🟢    | 🟢    | Suporte LibRaw (Genérico).                       | Suporte multicamadas (LibRaw + BruteForce JPEG).          | Ambas as versões sem thumbnails e preview, porem a v2 extraiu informações de dimensão e metadados corretamente. Esse formato parece ser da marca de camera leica                                          |

Notes sobre formatos Raw:

O formato **RAW** é um termo genérico que abrange centenas de formatos proprietários de diferentes fabricantes de câmeras (Canon .CR2/.CR3, Nikon .NEF, Sony .ARW, Fujifilm .RAF, Panasonic .RW2, etc.).

A tabela da V2 está correta:
*   **RAW**: O formato genérico não possui suporte nativo na V2.
*   **gpr** e **x3f**: São formatos específicos (GoPro e Sigma/Foveon) que possuem suporte nativo e funcional na V2, conforme validado na auditoria.

**Por que a V2 não tem suporte genérico para "RAW"?**
O suporte a RAW na V1 era fornecido pelo **dcraw**, uma biblioteca C mais antiga e de manutenção intermitente.
Na V2, optou-se por usar o **LibRaw**, que é a biblioteca padrão da indústria para processamento de RAWs modernos. O LibRaw não suporta todos os formatos genéricos de "RAW" que existiam no dcraw, mas cobre os principais e mais recentes.

Podemos observar o crate https://github.com/dnglab/dnglab/tree/main que tem suporte para grande parte dos formatos raw, escrito nativamente em rust.

### Video
| Manual Check | Extensão | V1   | V2   | V1 Notes                                  | V2 Notes                                                  | Manual chack notes                                                                                     |
| :----------- | :------- | :--- | :--- | :---------------------------------------- | :-------------------------------------------------------- | :----------------------------------------------------------------------------------------------------- |
| 🟢            | `3g2`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `3gp`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `asf`    | 🟢    | 🟢    | Estável via FFmpeg.                       | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `avi`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| ⚠️            | `divx`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |
| 🟢            | `f4v`    | 🟢    | 🟢    | Estável via FFmpeg.                       | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `flv`    | 🟢    | 🟢    | Estável via FFmpeg.                       | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| ⚠️            | `h264`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |
| ⚠️            | `h265`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |
| 🟢            | `hevc`   | 🟠    | 🟠    | Instabilidade no decodificador M3U8 local | Instabilidade no decodificador M3U8 local (Sprint 10.12). | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `m2ts`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `m2v`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `m4v`    | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback via Linear HLS.                         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `mjpeg`  | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| ⚠️            | `mjpg`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |
| 🟢            | `mkv`    | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback via Linear HLS.                         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `mov`    | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback via Linear HLS.                         | V2 extrai as dimensões corretamente coisa que a v1 parece não fazer                                    |
| 🟢            | `mp4`    | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback via Linear HLS.                         | V2 extrai as dimensões corretamente coisa que a v1 parece não fazer                                    |
| 🟢            | `mpeg`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 extrai as dimensões corretamente coisa que a v1 parece não fazer                                    |
| 🟢            | `mpg`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `mts`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `mxf`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `ogv`    | 🟠    | 🟢    | Transcode falha ou perde referência       | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar e não gerava thumbnail |
| ⚠️            | `qt`     | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback via Linear HLS.                         | Não testado                                                                                            |
| 🟢            | `rm`     | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar e não gerava thumbnail |
| ⚠️            | `rmvb`   | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |
| 🟢            | `swf`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `ts`     | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `vob`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `webm`   | 🟢    | 🟢    | Estável (FFmpeg).                         | Estável. Playback nativo/HLS.                             | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `wmv`    | 🟢    | 🟢    | Estável via FFmpeg.                       | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| 🟢            | `wtv`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | V2 parece ter suporte superior, a v1 fazia um transcode do video antes de tocar                        |
| ⚠️            | `y4m`    | 🟢    | 🟢    | Estável.                                  | Suporte completo via `VideoFormatProvider` + HLS.         | Não testado                                                                                            |

**Notas da verificação manual:** Apesar do suporte v2 parecer superior, a v1 oferece um ótimo suporte, houve erros pois ao testar foi rodado as aplicações com a arquitetura v1 em paralelo a v2, onde ao iniciar o servidor hls, a porta já estava sendo usada. Ao rodar a aplicação da v1 sozinha, tudo funcionou perfeitamente. é importante resaltar que a v2 está passando a informação correta de dimensão para a Viewport na frontend para todos os formatos, permitindo um calculo mais preciso em visualizações como mansory. Tambem foi alterado a porta para o servidor hls na v1 para teste e foi confirmado que o problema era conflito com as duas aplicações rodando em paralelo.


### Audio
| Manual Check | Extensão | V1   | V2   | V1 Notes                                   | V2 Notes                                      | Manual chack notes                                                                                                                                                                            |
| :----------- | :------- | :--- | :--- | :----------------------------------------- | :-------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🟢            | `aac`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend                                                                                 |
| ⚠️            | `aax`    | 🔴    | 🟢    | Codificação protegida ou erro de transcode | Transcoding via FFmpeg funcional na V2.       | Não testado                                                                                                                                                                                   |
| 🟢            | `ac3`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend                                                                                 |
| 🟢            | `aif`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend                                                                                 |
| 🟢            | `aifc`   | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend                                                                                 |
| 🟢            | `aiff`   | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend                                                                                 |
| 🟢            | `amr`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo                                                                                                                                                                                    |
| 🟢            | `ape`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo                                                                                                                                                                                    |
| ⚠️            | `bwf`    | 🔴    | 🟢    | Erro de transcode                          | Estável via `AudioFormatProvider`.            | Não testado                                                                                                                                                                                   |
| 🟢            | `caf`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `dts`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `flac`   | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🔴            | `m4a`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | A v2 apontou o erro "Failed to load media" na frontend                                                                                                                                        |
| 🟢            | `m4r`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo                                                                                                                                                                                    |
| 🔴            | `mid`    | 🔴    | 🟢    | Erro de transcode (falta Soundfont)        | Melhorado via transcoding HLS (Sprint 10.12). | Aconteceu um erro em ambas as versões por exemplo `FFprobe failed for "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/mid/bohemian_rhapsody.mid"` |
| 🔴            | `midi`   | 🔴    | 🟢    | Erro de transcode (falta Soundfont)        | Melhorado via transcoding HLS (Sprint 10.12). | Aconteceu um erro em ambas as versões por exemplo `FFprobe failed for "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/midi/bad_romance.midi"`     |
| 🟢            | `mka`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo                                                                                                                                                                                    |
| 🟢            | `mp2`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `mp3`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, mas aparentemente a leitura do formatu pela v1 foi mais rápida que a v2 e alguns waveforms não foram extraidos de ambas as versões                                                |
| 🟢            | `oga`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `ogg`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `opus`   | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `ra`     | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `spx`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `wav`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `wma`    | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |
| 🟢            | `wv`     | 🟢    | 🟢    | Estável.                                   | Estável. Playback nativo ou HLS.              | Tudo certo, porem em ambas as versões alguns arquivos não tiveram a waveform extraida ou mostrada no frontend, observei que acontece quando o arquivo é muito pequeno ou muito grande         |

### Project
| Manual Check | Extensão   | V1   | V2   | V1 Notes                                         | V2 Notes                                                    | Manual chack notes                                                                                                                                                                                                                                                 |
| :----------- | :--------- | :--- | :--- | :----------------------------------------------- | :---------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🟢🟢           | `afdesign` | 🟢    | 🟢    | Apenas thumbnail via assinatura PNG.             | Suporte completo via `AffinityFormatProvider`.              | v2 agora gera thumbnail, preview de alta qualidade em PNG e extrai metadados completos como largura, altura e resolução DPI do arquivo de preview integrado.                                                                                                       |
| 🟢🟢           | `afphoto`  | 🟢    | 🟢    | Apenas thumbnail via assinatura PNG.             | Suporte completo via `AffinityFormatProvider`.              | v2 agora gera thumbnail, preview de alta qualidade em PNG e extrai metadados completos como largura, altura e resolução DPI do arquivo de preview integrado.                                                                                                       |
| 🟢🟢           | `afpub`    | 🟢    | 🟢    | Apenas thumbnail via assinatura PNG.             | Suporte completo via `AffinityFormatProvider`.              | v2 agora gera thumbnail, preview de alta qualidade em PNG e extrai metadados completos como largura, altura e resolução DPI do arquivo de preview integrado.                                                                                                       |
| 🟢🟢           | `ai`       | 🟢    | 🟢    | Estável para arquivos baseados em PDF.           | Suporte completo (PDF e PostScript) via `AiFormatProvider`. | v2 extrai thumbnails de qualidade, dimensões exatas de canvas, resolução DPI e metadados.                                                                                                                                                                          |
| 🟢🟢           | `aseprite` | 🟢    | 🟢    | Estável.                                         | Metadados técnicos e semânticos completos.                  | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟢           | `ase`      | 🟢    | 🟢    | Alias estável.                                   | Metadados técnicos e semânticos completos.                  | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟢           | `clip`     | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `ClipStudioFormatProvider`.              | Implementação concluída com extração de metadata avançada aprimorada: agora extrai e relata as dimensões corretas do canvas (largura, altura) e a resolução (DPI) através da leitura e query da tabela 'Canvas' do banco de dados SQLite embutido no formato.      |
| 🟢🟢           | `kra`      | 🟢    | 🟢    | Estável via parsing de ZIP.                      | Nativo via `ProjectZipFormatProvider`.                      | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟢           | `mdp`      | 🟢    | 🟢    | Estável. Metadados e preview nativos.            | Suporte completo via `MedibangFormatProvider`.              | Tudo certo, metadados como dimensões, resolução e camadas agora são extraídos.                                                                                                                                                                                     |
| 🟢🟢           | `psd`      | 🟢    | 🟢    | Estável via `psd` crate.                         | Estável. Metadados e thumbnails nativos.                    | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟢           | `reb`      | 🟢    | 🟢    | Estável. Metadados avançados via `artwork.xml`.  | Suporte completo via `RebelleFormatProvider`.               | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟢           | `penpot`   | 🟢    | 🟢    | Estável via ZIP (V1) e Zstd (V2).                | Suporte nativo via `PenpotFormatProvider`.                  | Implementação concluída com extração de metadata avançada.                                                                                                                                                                                                         |
| 🟢🟢           | `rif`      | 🟢    | 🟢    | Suporte Corel Painter.                           | Suporte nativo via `CorelPainterFormatProvider`.            | Tudo certo, ambas as versões com thumb e preview de qualidade. Agora suporta extração de dimensões a partir da imagem do preview.                                                                                                                                  |
| 🟢🟢           | `sai`      | 🟢    | 🟢    | Parsing binário nativo para metadados e preview. | Suporte nativo via `PaintToolSaiFormatProvider`.            | Implementação concluída com nova arquitetura de metadata. As dimensões corretas do canvas são extraídas em ambas as versões.                                                                                                                                       |
| 🟢🟢           | `sai2`     | 🟢    | 🟢    | Parsing binário nativo para metadados e preview. | Suporte nativo via `PaintToolSaiFormatProvider`.            | V2 com suporte muito superior. Implementação concluída com nova arquitetura de metadata. As dimensões corretas do canvas são extraídas do header.                                                                                                                  |
| 🟢🟢           | `sketch`   | 🟢    | 🟢    | Estável. Metadados e preview nativos.            | Suporte nativo via `SketchFormatProvider`.                  | Implementação completa com extração de versão do app, páginas e dimensões do preview.                                                                                                                                                                              |
| 🟢🟢           | `xmind`    | 🟢    | 🟢    | Estável.                                         | Suporte nativo via `XMindFormatProvider`.                   | A implementação da v2 está mais completa                                                                                                                                                                                                                           |
| 🟢🟠           | `cdr`      | 🟢    | 🟢    | Estável. Metadados e preview nativos.            | Suporte nativo via `CoreldrawFormatProvider`.               | Implementação completa com suporte a todas as versões (v3-v24+), incluindo dimensões e versões.                                                                                                                                                                    |
| 🟢🟠           | `fig`      | 🟢    | 🟢    | Estável via parsing de ZIP.                      | Suporte nativo via `FigmaFormatProvider`.                   | Implementação completa com extração de dimensões via preview e suporte a comentários do container. Porem ainda é possível melhorar                                                                                                                                 |
| 🟢🟠           | `xcf`      | 🟢    | 🟢    | Problemas com modos de camada e máscaras         | Suporte nativo via `GimpFormatProvider`.                    | Implementação concluída com extração de metadados avançada aprimorada: agora extrai dimensões (largura, altura) e resolução física (DPI) do cabeçalho e propriedades `PROP_RESOLUTION` do GIMP. Modos de mesclagem complexos seguem como possível evolução futura. |
| 🔴            | `aep`      | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | v2 não indexou e assim como a v1, não tem thumbnail ou preview                                                                                                                                                                                                     |
| ⚠️            | `ari`      | 🟢    | 🟠    | Ícone nativo.                                    | Suporte via ícone genérico.                                 | Não testado                                                                                                                                                                                                                                                        |
| 🔴            | `braw`     | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | A v2 não indexou o arquivo e assim como a v1, não tem thumbnail ou preview, alem de identificar como vídeo.                                                                                                                                                        |
| ⚠️            | `drp`      | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `fcpxml`   | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `idml`     | 🟠    | 🟠    | Apenas ícone genérico/stub                       | Stub de ícone via `IconFormatProvider`.                     | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `indd`     | 🟠    | 🟠    | Apenas ícone genérico/stub                       | Stub de ícone via `IconFormatProvider`.                     | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `krz`      | 🟢    | 🟢    | Alias (Krita Compressed).                        | Nativo via `ProjectZipFormatProvider`.                      | Não testado                                                                                                                                                                                                                                                        |
| 🔴            | `prproj`   | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | v2 não indexou e assim como a v1, não tem thumbnail ou preview                                                                                                                                                                                                     |
| ⚠️            | `psb`      | 🟢    | 🟢    | Estável via `psd` crate.                         | Estável. Metadados e thumbnails nativos.                    | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `r3d`      | 🟠    | 🟠    | Stub mostra ícone genérico                       | Stub de ícone via `IconFormatProvider`.                     | Não testado                                                                                                                                                                                                                                                        |
| ⚠️            | `riff`     | 🟢    | 🟢    | Suporte Corel Painter.                           | Suporte nativo via `CorelPainterFormatProvider`.            | Não testado                                                                                                                                                                                                                                                        |

### Vector
| Manual check | Extensão | V1   | V2   | V1 Notes                                          | V2 Notes                                       | Manual check notes                                                                                                                                                                                    |
| :----------- | :------- | :--- | :--- | :------------------------------------------------ | :--------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🟢🟢           | `eps`    | 🔴    | 🟢    | Extração aborta (falta ponte Ghostscript estável) | Suporte nativo via `PostscriptFormatProvider`. | v2 agora extrai dimensões (altura e largura) com suporte a BoundingBox/HiResBoundingBox e conversão fallback para PDF.                                                                                |
| 🟢🟢           | `ps`     | 🔴    | 🟢    | Extração aborta (falta pnte Ghostscript estável)  | Suporte nativo via `PostscriptFormatProvider`. | v2 agora extrai dimensões (altura e largura) com suporte a BoundingBox/HiResBoundingBox e conversão fallback para PDF.                                                                                |
| 🟢🟢           | `svg`    | 🟢    | 🟢    | Nativo.                                           | Renderização via `resvg` e `tiny-skia`.        | v2 agora implementa MetadataCapability completa para SVG e SVGZ, extraindo largura, altura e resolução e resolvendo o erro de carregamento no frontend. SVGZ tem suporte nativo a descompressão gzip. |

### Archive

| Manual check | Extensão | V1   | V2   | V1 Notes                     | V2 Notes                                   | Manual check notes |
| :----------- | :------- | :--- | :--- | :--------------------------- | :----------------------------------------- | :----------------- |
|              | `7z`     | 🔴    | 🟠    | Não suportado.               | Apenas metadados, sem thumbnail (V2).      | .                  |
|              | `cbz`    | 🔴    | 🟢    | Apenas listagem de arquivos. | Suporte a extração de thumbnails internas. | .                  |
|              | `gz`     | 🔴    | 🟠    | Não suportado.               | Apenas metadados, sem thumbnail (V2).      | .                  |
|              | `rar`    | 🔴    | 🟠    | Não suportado.               | Apenas metadados, sem thumbnail (V2).      | .                  |
|              | `tar`    | 🔴    | 🟠    | Não suportado.               | Apenas metadados, sem thumbnail (V2).      | .                  |
|              | `zip`    | 🔴    | 🟢    | Apenas listagem de arquivos. | Suporte a extração de thumbnails internas. | .                  |

### Model3D
| Manual check | Extensão | V1   | V2   | V1 Notes                        | V2 Notes                                         | Manual check notes                                                                                                                                                 |
| :----------- | :------- | :--- | :--- | :------------------------------ | :----------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|              | `3ds`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    |                                                                                                                                                                    |
|              | `3mf`    | 🔴    | 🟢    | Não suportado.                  | Novo suporte via Assimp (V2).                    |                                                                                                                                                                    |
|              | `blend`  | 🟢    | 🟢    | Extração de preview REND block. | Nativo via `Model3dFormatProvider` (REND block). |                                                                                                                                                                    |
|              | `dae`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    |                                                                                                                                                                    |
| ⚠️            | `dwg`    | 🟢    | 🟢    | Suporte parcial (Autodesk SDK). | Registrado como `Model3D` (V2 Architecture).     | Não testado                                                                                                                                                        |
|              | `dxf`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    |                                                                                                                                                                    |
|              | `fbx`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    |                                                                                                                                                                    |
| 🟢            | `glb`    | 🟢    | 🟢    | Estável via Three.js/Assimp.    | Nativo via `Model3DFormatProvider`.              | Sem thumbnail porem com preview                                                                                                                                    |
|              | `gltf`   | 🟢    | 🟢    | Estável via Three.js/Assimp.    | Nativo via `Model3DFormatProvider`.              |                                                                                                                                                                    |
| ⚠️            | `iges`   | 🟢    | 🟢    | Metadados apenas.               | Suporte a metadados via `CadFormatProvider`.     | Não testado                                                                                                                                                        |
| ⚠️            | `igs`    | 🟢    | 🟢    | Metadados apenas.               | Suporte a metadados via `CadFormatProvider`.     | Não testado                                                                                                                                                        |
| 🟠            | `lwo`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    | A v1 gerava uma preview rapidamente, a v2 não gera preview pois ao abrir o ItemView, abre no visualizador de imagem. O formato está sendo identificado como imagem |
| 🟠            | `lws`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    | A v1 gerava uma preview rapidamente, a v2 não gera preview pois ao abrir o ItemView, abre no visualizador de imagem. O formato está sendo identificado como imagem |
| 🟠            | `obj`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    | A v1 gerava uma preview rapidamente, a v2 não gera preview                                                                                                         |
| ⚠️            | `sculpt` | 🟠    | 🟠    | Stub mostra ícone genérico      | Stub de ícone via `IconFormatProvider`.          | Não testado                                                                                                                                                        |
| ⚠️            | `step`   | 🟢    | 🟢    | Metadados apenas.               | Suporte a metadados via `CadFormatProvider`.     | Não testado                                                                                                                                                        |
| 🟠            | `stl`    | 🟢    | 🟢    | Suporte via Assimp.             | Conversão Assimp -> GLB (V2).                    | A v1 gerava uma preview rapidamente, a v2 não gera preview                                                                                                         |
| ⚠️            | `stp`    | 🟢    | 🟢    | Metadados apenas.               | Suporte a metadados via `CadFormatProvider`.     | Não testado                                                                                                                                                        |
| ⚠️            | `usd`    | 🟢    | 🟢    | Suporte nativo (macOS).         | Nativo via `UsdFormatProvider`.                  | Não testado                                                                                                                                                        |
| ⚠️            | `usda`   | 🟢    | 🟢    | Suporte nativo (macOS).         | Nativo via `UsdFormatProvider`.                  | Não testado                                                                                                                                                        |
| ⚠️            | `usdc`   | 🟢    | 🟢    | Suporte nativo (macOS).         | Nativo via `UsdFormatProvider`.                  | Não testado                                                                                                                                                        |
| ⚠️            | `usdz`   | 🟢    | 🟢    | Suporte nativo (macOS).         | Nativo via `UsdFormatProvider`.                  | Não testado                                                                                                                                                        |
| ⚠️            | `zpr`    | 🟠    | 🟠    | Stub mostra ícone genérico      | Stub de ícone via `IconFormatProvider`.          | Não testado                                                                                                                                                        |
| ⚠️            | `ztl`    | 🟠    | 🟠    | Stub mostra ícone genérico      | Stub de ícone via `IconFormatProvider`.          | Não testado                                                                                                                                                        |

### Font
| Manual check | Extensão | V1   | V2   | V1 Notes                              | V2 Notes                                 | Manual check notes                                                                          |
| :----------- | :------- | :--- | :--- | :------------------------------------ | :--------------------------------------- | :------------------------------------------------------------------------------------------ |
| 🔴            | `eof`    | 🔴    | 🔴    | Não suportado pela extração de glifos | Não suportado.                           |                                                                                             |
| 🟢🟢           | `otf`    | 🟢    | 🟢    | Estável.                              | Nativo via `OpenTypeFontProvider`.       | Estável. Corrigido regressão de thumbnail branca e adicionado suporte a fontes de símbolos. |
| 🟢🟢           | `ttc`    | 🟢    | 🟢    | Estável.                              | Nativo via `TrueTypeCollectionProvider`. | Estável. Corrigido regressão de thumbnail branca e adicionado extração de múltiplas faces.  |
| 🟢🟢           | `ttf`    | 🟢    | 🟢    | Estável.                              | Nativo via `TrueTypeFontProvider`.       | Estável. Corrigido regressão de thumbnail branca e adicionado suporte a fontes de símbolos. |
| 🟢🟢           | `woff`   | 🟢    | 🟢    | Estável.                              | Nativo via `WoffFontProvider`.           | Estável. Corrigido regressão de thumbnail branca e adicionado suporte a fontes de símbolos. |
| 🟢🟢           | `woff2`  | 🟢    | 🟢    | Estável.                              | Nativo via `Woff2FontProvider`.          | Estável. Corrigido regressão de thumbnail branca e adicionado suporte a fontes de símbolos. |

> Nota: Melhorar a preview no frontend para suportar fontes de símbolos.

### Documentos
| Manual check | Extensão | V1   | V2   | V1 Notes                       | V2 Notes                               | Manual check notes                                                                                                                                                                                                                 |
| :----------- | :------- | :--- | :--- | :----------------------------- | :------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🟢🟢           | `pdf`    | 🟢    | 🟢    | Nativo (Preview no navegador). | Preview e thumbnails nativos.          | v2 agora gera thumbnails da primeira página usando pdfium-render e extrai metadados completos como número de páginas, dimensões, autor, criador, produtor e datas de criação/modificação. PDF agora é classificado como documento. |
| 🔴            | `doc`    | 🟠    | 🟠    | Ícone apenas.                  | Stub de ícone (V2).                    |                                                                                                                                                                                                                                    |
| 🔴            | `docx`   | 🟠    | 🟠    | Ícone apenas.                  | Stub de ícone (V2).                    |                                                                                                                                                                                                                                    |
| 🔴            | `md`     | 🟢    | 🟢    | Renderização Markdown estável. | Suporte via `TextFormatProvider` (V2). |                                                                                                                                                                                                                                    |
| 🔴            | `txt`    | 🟢    | 🟢    | Preview de texto estável.      | Suporte via `TextFormatProvider` (V2). |                                                                                                                                                                                                                                    |
| 🔴            | `xls`    | 🟠    | 🟠    | Ícone apenas.                  | Stub de ícone (V2).                    |                                                                                                                                                                                                                                    |
| 🔴            | `xlsx`   | 🟠    | 🟠    | Ícone apenas.                  | Stub de ícone (V2).                    |                                                                                                                                                                                                                                    |

> Nota: Necessário a implementação de thumbnails e visualizador para documentos no frontend.

## Sem arquivos para testes

- **Imagens**: `avifs, `heifs`, `mjpg`
- **Áudio**: `aax`, `bwf`
- **Vídeo**: `divx`, `h264`, `h265`, `qt`, `rmvb`, `y4m`
- **Modelos 3D**: `dwg`, `iges`, `igs`, `sculpt`, `step`, `stp`, `usd`, `usda`, `usdc`, `usdz`, `zpr`, `ztl`
- **Projeto**: `ari`, `drp`, `fcpxml`, `idml`, `indd`, `krz`, `psb`, `r3d`, `riff`
- **Arquivo**: `7z`, `cbz`, `gz`, `tar`, `zip`
- **Fontes**: `eof`
- **Documentos**: `md`

## Formatos não registrados ou sem suporte

- **Imagens**: `cin`, `jp2`, `jps`, `pcd`, `pcx`, `picon`, `pict`, `ras`, `sgi`, `wbmp`, `wdp`, `xbm`, `xpm`, `xwd`
- **Áudio**: `8svx`, `aa`, `amb`, `au`, `avr`, `cdda`, `cvsd`, `fap`, `fssd`, `hcom`, `htk`, `ima`, `ircam`, `maud`, `nist`, `paf`, `pvf`, `sd2`, `smp`, `snd`, `sndr`, `sndt`, `sou`, `sph`, `tta`, `txw`, `voc`, `vox`, `w64`
- **Vídeo**: `dvms`, `mng`, `vms`, `wve`
- **Modelos 3D**: `3dm`, `bim`, `c4d`, `fts`, `iv`, `mtl`, `off`, `ply`, `skp`, `u3d`, `z3d`
- **Projeto**: `cpt` (corel photo paint)
- **Código**: `bat`, `c`, `class`, `cpp`, `cs`, `css`, `go`, `h`, `htaccess`, `html`, `java`, `js`, `json`, `pl`, `py`, `rb`, `sh`, `sln`, `sql`, `swift`, `yaml`
- **Fontes**: `afm`, `cff`, `dfont`, `eot`, `pfb`, `pfm`, `sfd`
- **Documentos**: `1st`, `azw3`, `csv`, `djvu`, `epub`, `fb2`, `inf`, `kml`, `kmz`, `lrf`, `mobi`, `odp`, `ods`, `odt`, `ott`, `pdb`, `ppt`, `rtf`, `snb`
- **Outros**: `bin`, `cfg`, `cvs`, `gsrt`, `jls`, `k25`, `map`, `mdc`, `nxt`, `pes`, `prc`, `test`, `vb`

