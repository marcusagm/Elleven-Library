import { createStore, reconcile } from 'solid-js/store';
import { untrack } from 'solid-js';
import { getImages } from '../../lib/db';
import { invoke } from '@tauri-apps/api/core';
import { tagService } from '../../lib/tags';
import { ActionResult, ErrorCode } from '../types/actions';
import { filterState, filterActions } from './filterStore';
import { selectionState } from './selectionStore';
import { type ImageItem } from '../../types';

export interface BatchChangeAddedItem extends ImageItem {
    folder_id: number;
    old_folder_id?: number;
}

export interface BatchChangeRemovedItem {
    id: number;
    folder_id: number;
    tag_ids: number[];
}

export interface BatchChangePayload {
    added?: BatchChangeAddedItem[];
    removed?: BatchChangeRemovedItem[];
    updated?: BatchChangeAddedItem[];
    needs_refresh?: boolean;
}

interface LibraryState {
    items: ImageItem[];
    isFetching: boolean;
    isRefreshing: boolean;
    totalItems: number; // useful for knowing if we reached end
}

import { APP_CONFIG } from '../../config/constants';

const BATCH_SIZE = APP_CONFIG.BATCH_SIZE;
let currentOffset = 0;

const [libraryState, setLibraryState] = createStore<LibraryState>({
    items: [],
    isFetching: false,
    isRefreshing: false,
    totalItems: 0
});

export const libraryActions = {
    /**
     * Internal helper to fetch a batch of images based on current filters.
     */
    fetchLibraryBatch: async (offset: number) => {
        const isUntagged = filterState.filterUntagged;
        const folderId = filterState.selectedFolderId;
        const recursive = filterState.folderRecursiveView;
        const anyFilter = filterActions.hasActiveFilters();
        const sortBy = filterState.sortBy;
        const sortOrder = filterState.sortOrder;

        const advancedQuery = filterState.advancedSearch
            ? JSON.stringify(filterState.advancedSearch)
            : undefined;

        if (anyFilter) {
            return await tagService.getImagesFiltered(
                BATCH_SIZE,
                offset,
                filterState.selectedTags,
                true,
                isUntagged,
                folderId || undefined,
                recursive,
                sortBy,
                sortOrder,
                advancedQuery,
                filterState.searchQuery
            );
        }
        return await getImages(BATCH_SIZE, offset, sortBy, sortOrder);
    },

    /**
     * Internal helper to async refresh the total items count.
     */
    refreshTotalCount: () => {
        const isUntagged = filterState.filterUntagged;
        const folderId = filterState.selectedFolderId;
        const recursive = filterState.folderRecursiveView;
        const anyFilter = filterActions.hasActiveFilters();

        const advancedQuery = filterState.advancedSearch
            ? JSON.stringify(filterState.advancedSearch)
            : undefined;

        if (anyFilter) {
            tagService
                .getImagesFilteredCount(
                    filterState.selectedTags,
                    true,
                    isUntagged,
                    folderId || undefined,
                    recursive,
                    advancedQuery,
                    filterState.searchQuery
                )
                .then(count => {
                    setLibraryState('totalItems', count);
                });
        } else {
            tagService
                .getImagesFilteredCount([], true, false, undefined, false, undefined, undefined)
                .then(count => {
                    setLibraryState('totalItems', count);
                });
        }
    },

    refreshImages: async (reset = false) => {
        if (libraryState.isRefreshing && reset) return;
        if (reset) setLibraryState('isRefreshing', true);

        try {
            const freshBatch = await libraryActions.fetchLibraryBatch(0);
            setLibraryState('items', reconcile(freshBatch, { key: 'id' }));
            currentOffset = BATCH_SIZE;

            libraryActions.refreshTotalCount();
        } finally {
            if (reset) setLibraryState('isRefreshing', false);
        }
    },

    loadMore: async () => {
        if (libraryState.isFetching) return;
        setLibraryState('isFetching', true);

        try {
            const nextBatch = await libraryActions.fetchLibraryBatch(currentOffset);

            if (nextBatch.length > 0) {
                setLibraryState('items', prev => [...prev, ...nextBatch]);
                currentOffset += BATCH_SIZE;
            }
        } finally {
            setLibraryState('isFetching', false);
        }
    },

    updateItemRating: async (id: number, rating: number) => {
        try {
            setLibraryState('items', i => i.id === id, 'rating', rating);
            await tagService.updateImageRating(id, rating);
        } catch (err) {
            console.error(`Failed to update rating for ${id}:`, err);
        }
    },

    updateItemNotes: async (id: number, notes: string) => {
        try {
            setLibraryState('items', i => i.id === id, 'notes', notes);
            await tagService.updateImageNotes(id, notes);
        } catch (err) {
            console.error(`Failed to update notes for ${id}:`, err);
        }
    },

    updateThumbnail: (id: number, path: string) => {
        setLibraryState('items', item => item.id === id, 'thumbnail_path', path);
    },

    handleBatchChange: (payload: BatchChangePayload) => {
        // 1. Handle Removals
        if (payload.removed && payload.removed.length > 0) {
            const removedIds = new Set(payload.removed.map(removedItem => removedItem.id));
            setLibraryState('items', items => items.filter(item => !removedIds.has(item.id)));
        }

        // 2. Handle Additions
        if (payload.added && payload.added.length > 0) {
            // Trigger a soft refresh to integrate new items in the correct order/position
            // reconcile will handle merging existing ones
            libraryActions.refreshImages(false);
        }

        // 3. Handle Updates (Moves and Renames)
        if (payload.updated && payload.updated.length > 0) {
            const updatedItems = payload.updated;
            import('./metadataStore').then(({ metadataState }) => {
                const { selectedFolderId, recursive, locations, currentItems } = untrack(() => ({
                    selectedFolderId: filterState.selectedFolderId,
                    recursive: filterState.folderRecursiveView,
                    locations: metadataState.locations,
                    currentItems: libraryState.items
                }));

                // Optimization: Create a Map for O(1) parent lookup instead of Array.find O(N)
                const locationMap = new Map(locations.map(location => [location.id, location]));

                const isChildOf = (childId: number, rootId: number): boolean => {
                    let current: number | null = childId;
                    let depth = 0;
                    // Use constant to prevent infinite loops (though DAG should prevent it)
                    while (current && depth < APP_CONFIG.MAX_FOLDER_DEPTH) {
                        if (current === rootId) return true;
                        const node = locationMap.get(current);
                        current = node ? node.parent_id : null;
                        depth++;
                    }
                    return false;
                };

                let someMovedIn = false;
                const toRemoveIDs: number[] = [];

                for (const item of updatedItems) {
                    const isNowInView =
                        !selectedFolderId ||
                        (recursive
                            ? isChildOf(item.folder_id, selectedFolderId)
                            : item.folder_id === selectedFolderId);

                    const wasKnown = currentItems.some(i => i.id === item.id);

                    if (isNowInView) {
                        if (wasKnown) {
                            // Update in place (Rename or Move within same recursive tree)
                            setLibraryState(
                                'items',
                                i => i.id === item.id,
                                prev => ({
                                    ...prev,
                                    path: item.path,
                                    filename: item.filename,
                                    modified_at: item.modified_at,
                                    folder_id: item.folder_id
                                })
                            );
                        } else {
                            // Moved INTO this folder view from outside
                            someMovedIn = true;
                        }
                    } else if (wasKnown) {
                        // Was here, but moved OUT
                        toRemoveIDs.push(item.id);
                    }
                }

                if (toRemoveIDs.length > 0) {
                    const removeSet = new Set(toRemoveIDs);
                    setLibraryState('items', items => items.filter(i => !removeSet.has(i.id)));
                }

                if (someMovedIn) {
                    // Re-fetch to get items moved in
                    libraryActions.refreshImages(false);
                }
            });
        }
    },

    setThumbnailPriority: async (ids: number[]) => {
        try {
            if (ids.length > 0) {
                await invoke('set_thumbnail_priority', { ids });
            }
        } catch (err) {
            console.error('Failed to set thumbnail priority:', err);
        }
    },

    /**
     * Adds a new root location to the library.
     * Triggers directory picker and starts indexing.
     */
    addLocation: async () => {
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const { metadataActions } = await import('./metadataStore');
            const { tauriService } = await import('../tauri/services');
            const { addLocation: dbAddLocation } = await import('../../lib/db');

            const selectedPath = await open({
                directory: true,
                multiple: false,
                title: 'Select Folder to Add'
            });

            if (selectedPath && !Array.isArray(selectedPath)) {
                await dbAddLocation(selectedPath);
                await metadataActions.loadLocations();
                await tauriService.startIndexing({ path: selectedPath });
                await libraryActions.refreshImages(true);
                return { success: true, path: selectedPath };
            }
            return { success: false };
        } catch (err) {
            console.error('Failed to add location:', err);
            return { success: false, error: err };
        }
    },

    /**
     * Removes a root location from the library.
     * @param locationId - The unique ID of the location to remove.
     */
    removeLocation: async (locationId: number) => {
        try {
            const { metadataActions } = await import('./metadataStore');
            await invoke('remove_location', { locationId });

            // Atomic refresh of related metadata
            await Promise.all([metadataActions.loadLocations(), metadataActions.loadStats()]);

            // Refresh library view
            await libraryActions.refreshImages(true);

            return { success: true };
        } catch (err) {
            console.error('Failed to remove location:', err);
            return { success: false, error: err };
        }
    },

    /**
     * Applies a specific tag to a batch of images.
     */
    applyTagToImages: async (
        imageIds: number[],
        tagId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        if (imageIds.length === 0) {
            return {
                success: false,
                error: { code: ErrorCode.VALIDATION_ERROR, message: 'No items provided' }
            };
        }

        try {
            const { metadataActions, metadataState } = await import('./metadataStore');
            await tagService.addTagsToImagesBatch(imageIds, [tagId]);
            await metadataActions.loadStats();

            // Refresh library view if filtering by tags
            if (filterState.selectedTags.length > 0) {
                await libraryActions.refreshImages(false);
            }

            const tagName = metadataState.tags.find(tag => tag.id === tagId)?.name || 'Tag';
            return {
                success: true,
                data: {
                    tagName,
                    count: imageIds.length
                }
            };
        } catch (error) {
            console.error('Failed to apply tags to images:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to apply tags' }
            };
        }
    },

    /**
     * Applies a specific tag to all currently selected images.
     */
    applyTagToSelection: async (
        tagId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        return libraryActions.applyTagToImages(selectionState.selectedIds, tagId);
    },

    /**
     * Intelligently applies a tag based on a drop target and current selection.
     */
    applyTagToTarget: async (
        tagId: number,
        targetImageId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        let targetIds = [targetImageId];
        if (selectionState.selectedIds.includes(targetImageId)) {
            targetIds = [...selectionState.selectedIds];
        }
        return libraryActions.applyTagToImages(targetIds, tagId);
    },

    /**
     * Removes a specific tag from all currently selected images.
     */
    removeTagFromSelection: async (tagId: number): Promise<ActionResult> => {
        const selectedIds = selectionState.selectedIds;
        if (selectedIds.length === 0) {
            return {
                success: false,
                error: { code: ErrorCode.VALIDATION_ERROR, message: 'No items selected' }
            };
        }

        try {
            const { metadataActions } = await import('./metadataStore');
            // tagService currently doesn't have a batch remove by tag ID,
            // we'll have to do it individually or update service.
            // For now, let's keep it simple as the backend usually handles singles better or we can extend it later.
            await Promise.all(selectedIds.map(id => tagService.removeTagFromImage(id, tagId)));
            await metadataActions.loadStats();
            // If we are filtering by tags, we might need a refresh
            if (filterState.selectedTags.length > 0) {
                await libraryActions.refreshImages(false);
            }
            return { success: true, data: undefined };
        } catch (error) {
            console.error('Failed to remove tags from selection:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to remove tags' }
            };
        }
    }
};

export { libraryState };
