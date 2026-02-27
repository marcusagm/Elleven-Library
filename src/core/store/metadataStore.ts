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

    /**
     * Notifies the system of a tag update.
     * @param options - Configuration for what needs refreshing.
     */
    notifyTagUpdate: (
        options: { structural?: boolean; stats?: boolean; images?: boolean } = {}
    ) => {
        const { structural = true, stats = true, images = true } = options;

        setMetadataState('tagUpdateVersion', v => v + 1);

        if (stats) {
            metadataActions.loadStats();
        }

        // Check if we need to refresh the library
        if (images) {
            import('./filterStore').then(({ filterState }) => {
                const isFilteringByTags =
                    filterState.filterUntagged || filterState.selectedTags.length > 0;
                if (isFilteringByTags) {
                    import('./libraryStore').then(({ libraryActions }) => {
                        libraryActions.refreshImages(structural); // If not structural, don't force re-fetch if possible
                    });
                }
            });
        }
    },

    loadTags: async () => {
        try {
            const tags = await tagService.getAllTags();
            // Sort by order_index primarily
            setMetadataState(
                'tags',
                tags.sort((a, b) => a.order_index - b.order_index)
            );
        } catch (error) {
            console.error('Failed to load tags:', error);
        }
    },

    /**
     * Creates a new tag and refreshes metadata.
     */
    createTag: async (
        name: string,
        parentId?: number | null,
        color?: string | null
    ): Promise<ActionResult<number>> => {
        try {
            const id = await tagService.createTag(name, parentId, color);
            await metadataActions.loadTags();
            metadataActions.notifyTagUpdate();
            return { success: true, data: id };
        } catch (error) {
            console.error('Failed to create tag:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to create tag' }
            };
        }
    },

    /**
     * Updates an existing tag and refreshes metadata.
     */
    updateTag: async (
        id: number,
        name?: string | null,
        color?: string | null,
        parentId?: number | null,
        orderIndex?: number | null
    ): Promise<ActionResult> => {
        try {
            await tagService.updateTag(
                id,
                name === null ? undefined : name,
                color === null ? undefined : color,
                parentId === null ? undefined : parentId,
                orderIndex === null ? undefined : orderIndex
            );

            // OPTIMIZATION: Update store locally instead of full loadTags()
            const tagUpdates: Partial<Tag> = {};
            if (name !== undefined && name !== null) tagUpdates.name = name;
            if (color !== undefined && color !== null) tagUpdates.color = color;
            if (parentId !== undefined) tagUpdates.parent_id = parentId === 0 ? null : parentId;
            if (orderIndex !== undefined && orderIndex !== null)
                tagUpdates.order_index = orderIndex;

            setMetadataState('tags', (tag: Tag) => tag.id === id, tagUpdates);

            // Determine if the change was "structural"
            // - parentId/orderIndex changes definitely are.
            // - name change might be if we sort by name or if it's used key titles.
            const isStructural = parentId !== undefined || orderIndex !== undefined;
            const nameChanged = name !== undefined && name !== null;

            if (isStructural) {
                // For structural changes, re-fetch to ensure sync with DB ordering/hierarchy
                await metadataActions.loadTags();
                metadataActions.notifyTagUpdate({ structural: true, stats: true, images: true });
            } else if (nameChanged) {
                // Name changed but not position
                metadataActions.notifyTagUpdate({ structural: false, stats: false, images: true });
            } else {
                // Visual change only (color)
                metadataActions.notifyTagUpdate({ structural: false, stats: false, images: false });
            }

            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to update tag:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to update tag' }
            };
        }
    },

    /**
     * Deletes a tag and all its descendants recursively.
     * @param id - The ID of the tag to delete.
     */
    deleteTagRecursive: async (id: number): Promise<ActionResult> => {
        try {
            const allTags = metadataState.tags;
            const toDelete = new Set<number>([id]);

            // Simple BFS to find all descendants
            const queue = [id];
            while (queue.length > 0) {
                const currentId = queue.shift()!;
                const children = allTags.filter(t => t.parent_id === currentId);
                for (const child of children) {
                    if (!toDelete.has(child.id)) {
                        toDelete.add(child.id);
                        queue.push(child.id);
                    }
                }
            }

            // Delete in chunks or parallel? Sequential for now to ensure consistency
            // if DB has constraints, though usually it's fine.
            for (const tagId of toDelete) {
                await tagService.deleteTag(tagId);
            }

            await metadataActions.loadTags();
            metadataActions.notifyTagUpdate();
            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to delete tags recursively:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to delete tag' }
            };
        }
    },

    /**
     * Reorders multiple tags in a single operation.
     */
    reorderTags: async (updates: { id: number; order: number }[]): Promise<ActionResult> => {
        try {
            // Apply updates sequentially to the DB
            await Promise.all(
                updates.map(u => tagService.updateTag(u.id, null, null, undefined, u.order))
            );
            await metadataActions.loadTags();
            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to reorder tags:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to reorder tags' }
            };
        }
    },

    /**
     * Moves a tag to a new parent or reorders it among siblings.
     */
    moveTag: async (
        draggedTagId: number,
        targetTagId: number | null,
        position: 'before' | 'inside' | 'after'
    ): Promise<ActionResult> => {
        try {
            const allTags = metadataState.tags;

            // 1. Resolve new parent
            let newParentId: number | null = null;
            if (position === 'inside') {
                newParentId = targetTagId;
            } else if (targetTagId !== null) {
                const targetTag = allTags.find(t => t.id === targetTagId);
                newParentId = targetTag ? targetTag.parent_id : null;
            }

            // 2. Build new sibling list
            const siblings = allTags
                .filter(tag => tag.parent_id === newParentId && tag.id !== draggedTagId)
                .sort((a, b) => a.order_index - b.order_index || a.name.localeCompare(b.name));

            let insertIndex = siblings.length;
            if (position !== 'inside') {
                const targetIndex = siblings.findIndex(tag => tag.id === targetTagId);
                if (targetIndex !== -1) {
                    insertIndex = position === 'before' ? targetIndex : targetIndex + 1;
                }
            }

            const draggedTag = allTags.find(t => t.id === draggedTagId);
            if (!draggedTag) throw new Error('Dragged tag not found');

            siblings.splice(insertIndex, 0, draggedTag);

            // 3. Create and execute updates
            const updates = siblings.map((tag, index) => {
                const newOrder = index * 100;
                const isDragged = tag.id === draggedTagId;

                // Only update if parent changed or order changed.
                // NOTE: We pass 0 for newParentId if it's null, as the Rust backend
                // interprets 0 as a signal to set parent_id to NULL.
                if (isDragged || tag.order_index !== newOrder) {
                    return tagService.updateTag(
                        tag.id,
                        null,
                        null,
                        isDragged ? (newParentId ?? 0) : tag.parent_id,
                        newOrder
                    );
                }
                return Promise.resolve();
            });

            await Promise.all(updates);
            await metadataActions.loadTags();

            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to move tag:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to move tag' }
            };
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
