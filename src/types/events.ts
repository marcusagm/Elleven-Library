// Auto-mapped from `src-tauri/src/core/events/payloads.rs` (DomainEvent enum)

export type DomainEvent =
    | { type: 'AssetCreated'; payload: { asset_id: string; path: string; format: string } }
    | { type: 'AssetTagsUpdated'; payload: { asset_id: string; active_tags: string[] } }
    | { type: 'AssetMetadataUpdated'; payload: { asset_id: string } }
    | {
          type: 'AssetStateChanged';
          payload: { asset_id: string; old_state: string; new_state: string };
      }
    | {
          type: 'FolderCreated';
          payload: { folder_id: string; parent_id: string | null; name: string; path: string };
      }
    | { type: 'AssetFolderChanged'; payload: { asset_id: string; folder_id: string | null } }
    | { type: 'FsFileDiscovered'; payload: { path: string; size_bytes: number } }
    | { type: 'FsPathDeleted'; payload: { path: string } }
    | { type: 'FsPathRenamed'; payload: { from: string; to: string } }
    | { type: 'ExtractionCompleted'; payload: { asset_id: string; capability: string } }
    | { type: 'JobFailed'; payload: { asset_id: string; error_reason: string } }
    | { type: 'ScanStarted'; payload: { library_id: string } }
    | { type: 'ScanCompleted'; payload: { library_id: string } }
    | { type: 'ThumbnailGenerated'; payload: { asset_id: string; path: string } };
