# Sprint 9.1: Correção de regressões e bugs gerados na migração do backend v1 para v2.

**Status da sprint:** Parcial
**Data e hora de inicio da sprint:** 2026-03-12 14:00
**Data e hora da conclusão da sprint:** -

## Tarefas

### Indexer e pastas

**Status:** Concluído
**Data e hora de inicio:** 2026-03-12 14:00
**Data e hora da conclusão:** 2026-03-12 18:00

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/db/folders.rs`
- `mundam-main/src-tauri/src/library/commands/folders.rs`
- `mundam-main/src-tauri/src/library/commands/indexing.rs`
- `mundam-main/src-tauri/src/indexer/mod.rs`
- `mundam-main/src-tauri/src/indexer/scan.rs`
- `mundam-main/src-tauri/src/indexer/watcher.rs`

**Lista de problemas**

- [x] Ao indexar, não está sendo mostrado mais a hierarquia de pastas da estrtutura real no disco rígido. Apenas a lista de pastas que foram adicionadas ao mundam. Onde deveria ser possível navegar pela hierarquia de pastas.
- [x] Antes ao adicionar uma pasta, o frontend mostrava o processo de inexação em `src/components/features/statusbar/StatusSystem.tsx`. Ao concluir, mostrava uma mensagem de conclusão.
- [x] Os assets apareciam assim que eram indexados na listagem em `src/components/features/viewport`
- [x] Anteriormente, era possível visualizar apenas os assets que uma pasta continha, sem considerar as subpastas, ou mostrar de forma recursiva, mostrando todos os assets de todas as subpastas. Agora ele mostra todos os assets de todas as pastas adicionadas ao mundam, sem considerar a hierarquia de pastas.
- [x] O contador de assets por pasta não está sendo mostrado corretamente em `src/components/features/library/LibrarySidebarPanel.tsx`. Ao indexar todos os arquivos, o contador continua exibindo `0` no lugar da contagem real. Provavelmente, esse erro ocorre devido a mudança do comando da API IPC que era `get_asset_count_filtered` para `get_library_stats`.


### Tags

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

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

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

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

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

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

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/library/commands/tags.rs`
- `mundam-main/src-tauri/src/db/search.rs`


**Lista de problemas**
- [ ] A alteração da ordenação de assets não está funcionando corretamente ao acionar `src/core/store/filter/filterState.tsx`

### Smart Folders e advanced search

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

**Arquivos da arquitetura v1 para referência:**
- `mundam-main/src-tauri/src/library/commands/smart_folders.rs`
- `mundam-main/src-tauri/src/library/commands/tags.rs`
- `mundam-main/src-tauri/src/db/search.rs`
- `mundam-main/src-tauri/src/db/models/smart_folder.rs`

**Lista de problemas**
- [ ] A busca por critérios não está funcionando.
  

### Settings

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

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

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

- [ ] Problemas com o color_worker. Alguns assets disparam erros como:
  ```shell
  2026-03-12T11:10:14.259671Z ERROR mundam_lib::processing::workers::color_worker: WORKER: Color analysis failed for asset 3ed1c21a-88ce-4454-9813-d5b82b632989: Internal state error: Failed to open thumbnail for color analysis: Format error decoding WebP: Invalid Chunk header: [52, 49, 46, 46]
  ```

## 💡 Notas para o Desenvolvedor / Agente

> Todos os formatos que eram suportados pelo backend v1, foram exaustivavente testados e todos estavam funcionando perfeitamente tanto na geração de thumbnails, quanto preview e extração de metadados. Precisamos que o backend v2 estja no mesmo nível ou superior, qualquer regressão é inadimissivel. 

## 🚀 Informações da Implementação

### Dificuldades e Desafios

#### Indexer e pastas

A migração inicial para V2 não contemplava a descoberta recursiva de pastas durante o `scan_directory`, salvando todos os assets no nível raiz da localização monitorada. Além disso, o sistema de eventos do V2 (Event Bus) não estava mapeado para os eventos legados do frontend (`indexer:progress`), resultando em uma barra de status estática. Outro desafio foi a performance do contador de assets recursivo, que foi resolvido com CTEs recursivas no SQLite.

Durante a implementação da descoberta da árvore de pastas, houve uma inconsistência de nomenclatura entre o comando esperado pelo frontend (`list_all_folders`) e o existente no backend (`get_all_subfolders`). Isso gerou erros de permissão no Tauri v2 que foram resolvidos através da consolidação dos comandos e ajuste nas capacidades de segurança.

#### Tags

Adicione informações sobre os problemas encontrados na tags.

#### ItemInspector e ItemView

Adicione informações sobre os problemas encontrados no ItemInspector e ItemView.

#### Listagem de assets e viewport

Adicione informações sobre os problemas encontrados na listagem de assets e viewport.

#### Smart Folders e advanced search

Adicione informações sobre os problemas encontrados na smart folders e advanced search.

#### Settings

Adicione informações sobre os problemas encontrados na settings.

### Melhorias Realizadas

#### Indexer e pastas

- **Escaneamento Hierárquico:** O `LibraryIndexer` agora detecta subpastas no disco e as cria automaticamente no banco de dados como entidades `Folder`.
- **Eventos de Progresso:** Implementada a emissão de eventos `ScanProgress` detalhados, com bridge no `lib.rs` para compatibilidade com o frontend.
- **Consultas Recursivas:** Adicionado suporte ao flag `recursive` no `AssetFilter` e implementação de CTEs recursivas no repositório para visualização de subpastas e contagem correta de assets.
- **Segurança e Permissões:** Configuração granulares de permissões no Tauri v2 para os novos comandos de query, garantindo que o frontend tenha acesso apenas ao necessário.
- **Consolidação de IPC:** Remoção de comandos duplicados e padronização de nomes entre frontend e backend para evitar regressões futuras.

#### Tags

Adicione informações sobre as melhorias realizadas na tags.

#### ItemInspector e ItemView

Adicione informações sobre as melhorias realizadas no ItemInspector e ItemView.

#### Listagem de assets e viewport

Adicione informações sobre as melhorias realizadas na listagem de assets e viewport.

#### Smart Folders e advanced search

Adicione informações sobre as melhorias realizadas na smart folders e advanced search.

#### Settings

Adicione informações sobre as melhorias realizadas na settings.

### 📄 Arquivos Criados ou Modificados

#### Indexer e pastas

- `src-tauri/src/core/models/asset.rs`: Adicionado flag `recursive` ao `AssetFilter`.
- `src-tauri/src/core/events/payloads.rs`: Adicionado evento `ScanProgress`.
- `src-tauri/src/infra/database/queries.rs`: Implementação de recursividade no SQL.
- `src-tauri/src/feature/library/indexer.rs`: Lógica de descoberta de hierarquia e emissão de progresso.
- `src-tauri/src/lib.rs`: Bridge de eventos e inicialização de serviços.
- `src-tauri/permissions/main.toml`: Definição de permissões para os comandos de pasta.
- `src-tauri/capabilities/default.json`: Atribuição de capacidades para o frontend.
- `src/lib/db.ts`: Chamada do comando `get_all_subfolders`.
- `src/core/store/library/libraryActions.ts`: Passagem do flag `recursive` para o backend.
- `src/types/index.ts`: Atualização da interface `AssetFilter`.

#### Tags

Adicione a lista de arquivos criados ou modificados na tags.

#### ItemInspector e ItemView

Adicione a lista de arquivos criados ou modificados no ItemInspector e ItemView.

#### Listagem de assets e viewport

Adicione a lista de arquivos criados ou modificados na listagem de assets e viewport.

#### Smart Folders e advanced search

Adicione a lista de arquivos criados ou modificados na smart folders e advanced search.

#### Settings

Adicione a lista de arquivos criados ou modificados na settings.
