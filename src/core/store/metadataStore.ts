/* eslint-disable max-lines */
import { createStore } from 'solid-js/store';
import { Tag, tagService } from '../../lib/tags';
import { getLocations } from '../../lib/db';
import { type BatchChangePayload } from './libraryStore';
import { computeStatsFromBatchChange } from './statsHelpers';
import { ActionResult, ErrorCode } from '../types/actions';
import { eventBus } from '../utils/eventBus';
import { metadataCache } from './metadata/cache';
import { type SearchGroup } from './filter';

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

/** Check if some added items belong to unknown folders */
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

        setMetadataState('tagUpdateVersion', version => version + 1);

        if (stats) {
            metadataActions.loadStats();
        }

        // Check if we need to refresh the library
        if (images) {
            import('./filter').then(({ filterState }) => {
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
        itemId: number,
        name?: string | null,
        color?: string | null,
        parentId?: number | null,
        orderIndex?: number | null
    ): Promise<ActionResult> => {
        try {
            const finalName = name === null ? undefined : name;
            const finalColor = color === null ? undefined : color;
            const finalParentId = parentId === null ? undefined : parentId;
            const finalOrderIndex = orderIndex === null ? undefined : orderIndex;

            await tagService.updateTag(
                itemId,
                finalName,
                finalColor,
                finalParentId,
                finalOrderIndex
            );

            metadataActions.applyTagStoreUpdates(itemId, name, color, parentId, orderIndex);
            metadataActions.resolveTagNotificationType(name, parentId, orderIndex);

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
     * Applies local store updates after a tag modification.
     */
    applyTagStoreUpdates: (
        itemId: number,
        name?: string | null,
        color?: string | null,
        parentId?: number | null,
        orderIndex?: number | null
    ) => {
        const tagUpdates: Partial<Tag> = {};

        if (name !== null && name !== undefined) tagUpdates.name = name;
        if (color !== null && color !== undefined) tagUpdates.color = color;
        if (orderIndex !== null && orderIndex !== undefined) tagUpdates.order_index = orderIndex;

        if (parentId !== undefined) {
            tagUpdates.parent_id = parentId === 0 || parentId === null ? null : parentId;
        }

        setMetadataState('tags', (tag: Tag) => tag.id === itemId, tagUpdates);
    },

    /**
     * Determines and triggers the correct notification after a tag update.
     */
    resolveTagNotificationType: async (
        name?: string | null,
        parentId?: number | null,
        orderIndex?: number | null
    ) => {
        const isStructuralChange = parentId !== undefined || orderIndex !== undefined;

        if (isStructuralChange) {
            await metadataActions.loadTags();
            metadataActions.notifyTagUpdate({ structural: true });
        } else {
            metadataActions.notifyTagUpdate({
                structural: false,
                stats: false,
                images: name !== null && name !== undefined
            });
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
    },

    /**
     * Batch updates tags for multiple assets.
     */
    updateAssetsTags: async (
        assetIds: number[],
        tagIds: number[],
        mode: 'merge' | 'replace' | 'remove'
    ): Promise<ActionResult> => {
        try {
            if (mode === 'merge') {
                await tagService.addTagsToImagesBatch(assetIds, tagIds);
            } else if (mode === 'remove') {
                await tagService.removeTagsFromImagesBatch(assetIds, tagIds);
            } else {
                await tagService.replaceTagsForImagesBatch(assetIds, tagIds);
            }

            metadataActions.notifyTagUpdate({ stats: true, images: true });
            eventBus.emit('metadata:changed', { type: 'tag', ids: tagIds });

            return { success: true, data: undefined };
        } catch (error) {
            console.error('Batch tag update failed:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Batch tag update failed' }
            };
        }
    },

    /**
     * Batch updates metadata (rating, notes) for multiple assets.
     */
    updateAssetsMetadata: async (
        assetIds: number[],
        metadata: { rating?: number; notes?: string }
    ): Promise<ActionResult> => {
        try {
            const updates = assetIds.flatMap(id => {
                const results = [];
                if (metadata.rating !== undefined) {
                    results.push(tagService.updateImageRating(id, metadata.rating));
                }
                if (metadata.notes !== undefined) {
                    results.push(tagService.updateImageNotes(id, metadata.notes));
                }
                return results;
            });

            await Promise.all(updates);

            metadataActions.notifyTagUpdate({ stats: false, images: true });
            eventBus.emit('assets:metadata-updated', {
                assetIds: assetIds.map(String),
                fields: Object.keys(metadata)
            });

            return { success: true, data: undefined };
        } catch (error) {
            console.error('Batch metadata update failed:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Batch metadata update failed' }
            };
        }
    },

    /**
     * Retrieves EXIF/Technical metadata for an asset, utilizing the local cache.
     */
    getAssetExif: async (assetId: number, path: string): Promise<Record<string, string>> => {
        const cached = metadataCache.get<Record<string, string>>(String(assetId));
        if (cached) return cached;

        try {
            const exif = await tagService.getImageExif(path);
            metadataCache.set(String(assetId), exif);
            return exif;
        } catch (error) {
            console.error(`Failed to load EXIF for asset ${assetId}:`, error);
            return {};
        }
    },

    /**
     * Retrieves the tags associated with a specific asset.
     */
    getAssetTags: async (assetId: number): Promise<Tag[]> => {
        try {
            return await tagService.getTagsForImage(assetId);
        } catch (error) {
            console.error(`Failed to load tags for asset ${assetId}:`, error);
            return [];
        }
    }
};

export { metadataState };
