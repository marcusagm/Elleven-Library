# Tauri IPC Contracts (Contratos JSON Entre Fronteiras)

Na nova arquitetura do Mundam, a fronteira entre o Frontend (React/Solid) e o Backend (Rust) é a linha mais sagrada. É ela que permite reconstruir todo o interior do Backend sem quebrar a UI, bastando respeitar as Assinaturas de Tipos aqui estipuladas.

Essa documentação serve como o "Swagger/OpenAPI" interno do Tauri.

---

## 1. Tratamento Global de Erros (O Padrão)

Não devolvemos `Error` crus de lib para o Frontend (panic ou strings soltas). Todo Comando Tauri (`Result<T, AppError>`) que der erro retornará um objeto tipado para a UI, viabilizando tratamentos granulares (Toasts apropriados).

```json
// Objeto de Rejeição de Comando:
{
  "code": "ASSET_NOT_FOUND",  // "FS_ACCESS_DENIED", "VALIDATION_FAILED", "DB_LOCKED_TIMEDOUT"
  "message": "The requested asset ID was not found in the database.",
  "details": "Asset ID: 52a1-bdc2-99ff" // Propriedade opcional de debug
}
```

---

## 2. CQRS: As Queries (Operações de Leitura - Fast Path)

Queries têm permissão arquitetural de "furar a fila" e ir direto ao Banco de Dados (SQLite Adapter), lendo os Read Models em milissegundos. Elas NUNCA alteram estado.

### 2.1 `get_assets_filtered`
Puxa a grade visual principal da biblioteca. Usado infinitamente em Scroll Virtual.

**Payload de Envio (Request):**
```ts
interface GetAssetsFilterReq {
    filters: {
        families?: string[];          // ["IMAGE", "VIDEO"]
        tags_include?: string[];      // ["3d_render", "character"]
        search_query?: string;        // "guerreiro medieval"
        colors?: string[];            // ["#ff0000"]
    };
    pagination: {
        limit: number;
        offset: number;
    };
    sort_by: "created_at" | "name" | "file_size" | "duration";
    sort_desc: boolean;
}
```

**Payload de Retorno (Response):**
```ts
interface GetAssetsRes {
    total_count: number;
    items: Array<{
        id: string;
        name: string;
        path: string;            // Para invocar thumbnails via asset:// protocol
        format_type: string;     // Para decidir qual ícone exibir num PDF, p.ex.
        family: string;
        width: number | null;
        height: number | null;
        dominant_colors: string[]; // ['#FF0000', '#AABBCC']
        tags: Array<{ id: string, name: string }>; 
    }>;
}
```

---

## 3. CQRS: Os Commands (Operações de Escrita - Ledger Path)

Mutações nunca vão direto para o Banco. Elas são empacotadas no Frontend como Intenções ("Commands") e submetidas ao `CommandHandlers`, que joga para o `AssetLedger`.

### 3.1 `update_asset_tags`
Comando disparado quando o usuário clica numa bolha de Tag na interface para vincular ou desvincular de fotos.

**Payload de Envio (Request):**
```ts
interface UpdateAssetTagsCommand {
    asset_ids: string[];          // Suporta Bulk Operation (Múltiplas fotos)
    add_tags: string[];           // IDs das Tags
    remove_tags: string[];        // IDs das Tags
}
```

**Payload de Retorno (Response):**
O Ledger responde de forma sincrona apenas se a intenção foi Aceita. O Frontend não recebe o Asset alterado aqui. Recebe um OK e confia no *Domain Event* que atualizará o Estado global.
```json
{
  "success": true,
  "transaction_id": "tx_fa9bc29z"
}
```

### 3.2 `reindex_directory` (Escaneamento Nativo)
Comanda o Watcher Ativo a repassar pente-fino forçado numa sub-pasta.

**Payload de Envio:**
```ts
{ "path_to_scan": "/Users/Artist/References/Poses" }
```

---

## 4. O Sistema de Streaming Reativo (Eventos Backend -> Frontend)

Não bastam respostas HTTP-like. A essência do DAM dinâmico dita que quando um Sub-Processador invisível termina algo, a UI reage sozinha (Sincronização 2-Way).

### 4.1 Ouvinte de Status Vital da Galeria (`asset_updated_event`)

Enviado pelo `EventBus` Rust -> `emit()` Tauri. Quando os Trabalhadores de FFmpeg (`processing/transcoding`) ou o Atualizador do Ledger terminam de salvar a *Thumbnail* que não existia.

**Payload do Evento recebido no Front:**
```ts
interface AssetUpdatedEvent {
    asset_id: string;
    update_type: "THUMBNAIL_READY" | "METADATA_EXTRACTED" | "TAGS_CHANGED" | "RENAMED_ON_FS";
    partial_payload?: {
        // Ex: as novas cores Hex ou o novo Path
        // A UI pode aplicar 'optimistic update' na Grade de Itens sem fazer Fetch inteiro!
    }
}
```

### 4.2 Ouvinte do Progresso Havy-Duty (`indexer_progress_event`)

```ts
interface IndexerProgressEvent {
    root_folder_id: string;
    total_files_discovered: number;
    files_processed: number;
    current_file_path: string;  
    status: "SCANNING" | "EXTRACTING" | "COMPLETED" | "FAILED";
}
```
A barra de carregamento global do Solid.js se liga única e exclusivamente a essa assinatura de JSON!
