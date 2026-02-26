import { createStore } from 'solid-js/store';
import { Tag, tagService } from '../../lib/tags';
import { getLocations } from '../../lib/db';
import { type BatchChangePayload } from './libraryStore';
import { type SearchGroup } from './filterStore';
import { computeStatsFromBatchChange } from './statsHelpers';
import { ActionResult, ErrorCode } from '../types/actions';

interface FolderNode {
    id: number;
    path: string;
    name: string;
    parent_id: number | null;
    is_root: boolean;
}

export interface SmartFolder {
    id: number;
    name: string;
    query_json: string;
    created_at: string;
}

interface MetadataState {
    tags: Tag[];
    locations: FolderNode[];
    smartFolders: SmartFolder[];
    libraryStats: {
        total_images: number;
        untagged_images: number;
        tag_counts: Map<number, number>;
        folder_counts: Map<number, number>;
        folder_counts_recursive: Map<number, number>;
    };
    tagUpdateVersion: number;
}

const [metadataState, setMetadataState] = createStore<MetadataState>({
    tags: [],
    locations: [],
    smartFolders: [],
    libraryStats: {
        total_images: 0,
        untagged_images: 0,
        tag_counts: new Map(),
        folder_counts: new Map(),
        folder_counts_recursive: new Map()
    },
    tagUpdateVersion: 0
});

/** Check if any added items belong to unknown folders */
function hasUnknownFolders(added: BatchChangePayload['added'], knownIds: Set<number>): boolean {
    if (!added) return false;
    return added.some(item => item.folder_id && !knownIds.has(item.folder_id));
}

export const metadataActions = {
    loadSmartFolders: async () => {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            const folders = (await invoke('get_smart_folders')) as SmartFolder[];
            setMetadataState('smartFolders', folders);
        } catch (error) {
            console.error('Failed to load smart folders:', error);
        }
    },

    saveSmartFolder: async (
        name: string,
        query: SearchGroup | null,
        id?: number
    ): Promise<ActionResult> => {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            if (id) {
                await invoke('update_smart_folder', { id, name, query: JSON.stringify(query) });
            } else {
                await invoke('save_smart_folder', { name, query: JSON.stringify(query) });
            }
            await metadataActions.loadSmartFolders();
            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to save smart folder:', error);
            return {
                success: false,
                error: {
                    code: ErrorCode.IO_ERROR,
                    message: 'Failed to save smart folder'
                }
            };
        }
    },

    deleteSmartFolder: async (id: number): Promise<ActionResult> => {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('delete_smart_folder', { id });
            await metadataActions.loadSmartFolders();
            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to delete smart folder:', error);
            return {
                success: false,
                error: {
                    code: ErrorCode.IO_ERROR,
                    message: 'Failed to delete smart folder'
                }
            };
        }
    },

    notifyTagUpdate: () => {
        setMetadataState('tagUpdateVersion', v => v + 1);
        metadataActions.loadStats();

        // Check if we need to refresh the library
        import('./filterStore').then(({ filterState }) => {
            if (filterState.filterUntagged || filterState.selectedTags.length > 0) {
                import('./libraryStore').then(({ libraryActions }) => {
                    libraryActions.refreshImages(true);
                });
            }
        });
    },

    loadTags: async () => {
        try {
            const tags = await tagService.getAllTags();
            setMetadataState('tags', tags);
        } catch (error) {
            console.error('Failed to load tags:', error);
        }
    },

    loadLocations: async () => {
        try {
            const locations = await getLocations();
            setMetadataState('locations', locations);
        } catch (error) {
            console.error('Failed to load locations:', error);
        }
    },

    loadStats: async () => {
        try {
            const stats = await tagService.getLibraryStats();
            const tagMap = new Map();
            stats.tag_counts.forEach(c => tagMap.set(c.tag_id, c.count));

            const folderMap = new Map();
            stats.folder_counts.forEach(c => folderMap.set(c.folder_id, c.count));

            const folderRecursiveMap = new Map();
            if (stats.folder_counts_recursive) {
                stats.folder_counts_recursive.forEach(c =>
                    folderRecursiveMap.set(c.folder_id, c.count)
                );
            }

            setMetadataState('libraryStats', {
                total_images: stats.total_images,
                untagged_images: stats.untagged_images,
                tag_counts: tagMap,
                folder_counts: folderMap,
                folder_counts_recursive: folderRecursiveMap
            });
        } catch (error) {
            console.error('Failed to load library stats:', error);
        }
    },

    refreshAll: async () => {
        await Promise.all([
            metadataActions.loadTags(),
            metadataActions.loadLocations(),
            metadataActions.loadStats(),
            metadataActions.loadSmartFolders()
        ]);
    },

    handleBatchChange: (payload: BatchChangePayload) => {
        const knownIds = new Set(metadataState.locations.map(location => location.id));
        let needsRefresh = payload.needs_refresh ?? false;

        if (hasUnknownFolders(payload.added, knownIds)) {
            needsRefresh = true;
        }

        setMetadataState('libraryStats', stats => {
            const result = computeStatsFromBatchChange(
                stats,
                payload,
                metadataState.locations,
                knownIds
            );
            needsRefresh = needsRefresh || result.needsRefresh;
            return result.newStats;
        });

        if (needsRefresh) {
            metadataActions.refreshAll();
        }
    }
};

export { metadataState };
