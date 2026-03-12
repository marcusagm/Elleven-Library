# Sprint 9.1: Correção de regressões e bugs gerados na migração do backend v1 para v2.

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

## Tarefas

### Indexer e pastas

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/db/folders.rs`
- `mundam-main/src-tauri/src/library/commands/folders.rs`
- `mundam-main/src-tauri/src/library/commands/indexing.rs`
- `mundam-main/src-tauri/src/indexer/mod.rs`
- `mundam-main/src-tauri/src/indexer/scan.rs`
- `mundam-main/src-tauri/src/indexer/watcher.rs`

**Lista de problemas**

- [ ] Ao indexar, não está sendo mostrado mais a hierarquia de pastas da estrtutura real no disco rígido. Apenas a lista de pastas que foram adicionadas ao mundam. Onde deveria ser possível navegar pela hierarquia de pastas.
- [ ] Antes ao adicionar uma pasta, o frontend mostrava o processo de inexação em `src/components/features/statusbar/StatusSystem.tsx`. Ao concluir, mostrava uma mensagem de conclusão.
- [ ] Os assets apareciam assim que eram indexados na listagem em `src/components/features/viewport`
- [ ] Anteriormente, era possível visualizar apenas os assets que uma pasta continha, sem considerar as subpastas, ou mostrar de forma recursiva, mostrando todos os assets de todas as subpastas. Agora ele mostra todos os assets de todas as pastas adicionadas ao mundam, sem considerar a hierarquia de pastas.
- [ ] O contador de assets por pasta não está sendo mostrado corretamente em `src/components/features/library/LibrarySidebarPanel.tsx`. Ao indexar todos os arquivos, o contador continua exibindo `0` no lugar da contagem real. Provavelmente, esse erro ocorre devido a mudança do comando da API IPC que era `get_asset_count_filtered` para `get_library_stats`.


### Tags

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/db/tags.rs`
- `mundam-main/src-tauri/src/library/commands/tags.rs`

**Lista de problemas**

- [ ] Ao criar clicar para criar uma nova tag em `src/components/features/tags/TagTreeSidebarPanel.tsx` deveria ficar um campo de texto para digitar o nome da tag e apertar enter para criar. Atualmente ele cria a tag, mas é preciso ir no menu de contexto para renomear.
- [ ] As tags não permitem mais a definição hierarquica igual o backend v1 permitia. Onde era possível criar tags filhas de outras tags. Agora ele cria tags como se fossem todas do mesmo nível.
- [ ] Os seguintes erros ocorrem ao tentar usar o DND para atribuir uma tag a um asset, seja arrastando a tag para um asset, arrastando um ou mais assets selecionados para uma tag, ou atribuindo via `TagInput` pelo `src/components/features/inspector/base/InspectorTags.tsx`:
  ```shell
  [Error] [IPC Error: add_tags_to_assets_batch] – "invalid args `assetIds` for command `add_tags_to_assets_batch`: command add_tags_to_assets_batch missing required key assetIds"
	(anonymous function) (api.ts:30)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  [Error] Batch tag update failed: – "invalid args `assetIds` for command `add_tags_to_assets_batch`: command add_tags_to_assets_batch missing required key assetIds"
	(anonymous function) (tagActions.ts:233)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  ```
- [ ] Ao tentar reordenar ou adicionar uma tag como filha de outra tag usando o DND, ocorre o seguinte erro:
  ```shell
  [Error] Failed to move tag:
  Error: Dragged tag not found
  (anonymous function) — tagActions.ts:231
  asyncFunctionResume
  (anonymous function)
  promiseReactionJobWithoutPromise
  promiseReactionJob
	(anonymous function) (tagActions.ts:210)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  ```
  

### ItemInspector

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/formats/definitions.rs`
- `mundam-main/src-tauri/src/library/commands/formats.rs`
- `mundam-main/src-tauri/src/library/commands/metadata.rs`

**Lista de problemas**

- [ ] Vários formatos de assets não estão abrindo o inspector correto de acordo com o seu tipo. Por exemplo, arquivos `dts`, `aif` e `ac3` de audio deveriam carregar `src/components/features/inspector/audio`, arquivo `f4v`, `mjpeg`, `asf` de video deveriam carregar `src/components/features/inspector/video`, etc. Porem esses arquivos carregam o inspector de imagem `src/components/features/inspector/image`.
- [ ] O player de video não está funcionando. Ao clicar para reproduzir um video, ele não inicia a reprodução `src/components/features/inspector/video`.
- [ ] O player de audio não está funcionando. Ao clicar para reproduzir um audio, ele não inicia a reprodução `src/components/features/inspector/audio`.
- [ ] Ao selecionar um arquivo é comum ocorrer o erro a seguir quando o tipo de asset não possui EXIF:
  ```shell
  [Error] [IPC Error: get_asset_exif] – {code: "INTERNAL_ERROR", message: 'Application error: Provider for "/Users/marcusmaia…e-Point.kra" does not support metadata extraction'}
  {code: "INTERNAL_ERROR", message: 'Application error: Provider for "/Users/marcusmaia…e-Point.kra" does not support metadata extraction'} Object
	(anonymous function) (api.ts:30)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob

   [Error] Failed to load EXIF for asset 1561aad7-e7f9-4a7a-a2e3-657718b54ece: – {code: "INTERNAL_ERROR", message: "Application error: Provider for \"/Users/marcusmaia…e-Point.kra\" does not support metadata extraction"}
   {code: "INTERNAL_ERROR", message: "Application error: Provider for \"/Users/marcusmaia…e-Point.kra\" does not support metadata extraction"}Object
	(anonymous function) (tagActions.ts:281)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  ```

### ItemView

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/formats/definitions.rs`
- `mundam-main/src-tauri/src/library/commands/formats.rs`
- `mundam-main/src-tauri/src/library/commands/metadata.rs`

**Lista de problemas**

- [ ] Vários formatos de assets não estão abrindo o ItemView correto de acordo com o seu tipo. Por exemplo, arquivos `dts`, `aif` e `ac3` de audio deveriam carregar `src/components/features/itemview/renderers/audio`, arquivo `f4v`, `mjpeg`, `asf` de video deveriam carregar `src/components/features/itemview/renderers/video`, etc. Porem esses arquivos carregam o ItemView de imagem `src/components/features/itemview/renderers/image`.
- [ ] O player de video não está funcionando. Ao clicar para reproduzir um video, ele não inicia a reprodução `src/components/features/itemview/renderers/video`.
- [ ] O player de audio não está funcionando. Ao clicar para reproduzir um audio, ele não inicia a reprodução `src/components/features/itemview/renderers/audio`.
- [ ] Ao abrir alguns arquivos para visualização, o backend v1 possuia métodos específicos para visualização em tamanho completo. Onde ele carregava um preview do arquivo original no tamanho real. Alguns arquivos não conseguem ser abertos por causa disso.
- [ ] Agora ao tentar abrir um arquivo, ocorre o erro para alguns arquivos:
  ```shell
  [Error] Failed to load EXIF for asset c06d6751-0621-4dc2-8b43-765bd6919002: – {code: "INTERNAL_ERROR", message: "Application error: Provider for \"/Users/marcusmaia…r_Huion.kra\" does not support metadata extraction"}
  {code: "INTERNAL_ERROR", message: "Application error: Provider for \"/Users/marcusmaia…r_Huion.kra\" does not support metadata extraction"}Object
	(anonymous function) (tagActions.ts:281)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  ```

### Listagem de assets e viewport

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/library/commands/tags.rs`
- `mundam-main/src-tauri/src/db/search.rs`


**Lista de problemas**
- [ ] A alteração da ordenação de assets não está funcionando corretamente ao acionar `src/core/store/filter/filterState.tsx`

### Smart Folders e advanced search

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/library/commands/smart_folders.rs`
- `mundam-main/src-tauri/src/library/commands/tags.rs`
- `mundam-main/src-tauri/src/db/search.rs`
- `mundam-main/src-tauri/src/db/models/smart_folder.rs`

**Lista de problemas**
- [ ] A busca por critérios não está funcionando.
  

### Settings

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/settings/commands.rs`
- `mundam-main/src-tauri/src/transcoding/commands.rs`

**Lista de problemas**
- [ ] O comando `get_cache_stats` não está funcionando.
  ```shell
  [Error] [IPC Error: get_cache_stats] – "Command get_cache_stats not found"
	(anonymous function) (api.ts:30)
	asyncFunctionResume
	(anonymous function)
	promiseReactionJobWithoutPromise
	promiseReactionJob
  ```


### Erros no terminal

- [ ] Problemas com o color_worker. Alguns assets disparam erros como:
  ```shell
  2026-03-12T11:10:14.259671Z ERROR mundam_lib::processing::workers::color_worker: WORKER: Color analysis failed for asset 3ed1c21a-88ce-4454-9813-d5b82b632989: Internal state error: Failed to open thumbnail for color analysis: Format error decoding WebP: Invalid Chunk header: [52, 49, 46, 46]
  ```

## 💡 Notas para o Desenvolvedor / Agente

> Todos os formatos que eram suportados pelo backend v1, foram exaustivavente testados e todos estavam funcionando perfeitamente tanto na geração de thumbnails, quanto preview e extração de metadados. Precisamos que o backend v2 estja no mesmo nível ou superior, qualquer regressão é inadimissivel. 

## 🚀 Informações da Implementação

### Dificuldades e Desafios

-

### Melhorias Realizadas

-

### 📄 Arquivos Criados ou Modificados

-
