import { reconcile } from 'solid-js/store';
import { untrack } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { getAssets } from '../../../lib/db';
import { tagService } from '../../../lib/tags';
import { ActionResult, ErrorCode } from '../../types/actions';
import { filterState, filterActions } from '../filter';
import { selectionState } from '../selectionStore';
import { libraryState, libraryStateInternal } from './libraryState';
import { BatchChangePayload } from './schemas';
import { APP_CONFIG } from '../../../config/constants';

const { setLibraryState } = libraryStateInternal;
const BATCH_SIZE = APP_CONFIG.BATCH_SIZE;
let currentOffset = 0;

import { itemActions } from './itemActions';

export const libraryActions = {
    ...itemActions,

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
            return await tagService.getAssetsFiltered(
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
        return await getAssets(BATCH_SIZE, offset, sortBy, sortOrder);
    },

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
                .getAssetsFilteredCount(
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
                .getAssetsFilteredCount([], true, false, undefined, false, undefined, undefined)
                .then(count => {
                    setLibraryState('totalItems', count);
                });
        }
    },

    refreshAssets: async (reset = false) => {
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

    handleBatchChange: (payload: BatchChangePayload) => {
        if (payload.removed && payload.removed.length > 0) {
            const removedIds = new Set(payload.removed.map(removedItem => removedItem.id));
            setLibraryState('items', items => items.filter(item => !removedIds.has(item.id)));
        }

        if (payload.added && payload.added.length > 0) {
            libraryActions.refreshAssets(false);
        }

        if (payload.updated && payload.updated.length > 0) {
            const updatedItems = payload.updated;
            import('../metadata').then(({ metadataState }) => {
                const { selectedFolderId, recursive, locations, currentItems } = untrack(() => ({
                    selectedFolderId: filterState.selectedFolderId,
                    recursive: filterState.folderRecursiveView,
                    locations: metadataState.locations,
                    currentItems: libraryState.items
                }));

                const locationMap = new Map(locations.map(location => [location.id, location]));

                const isChildOf = (childId: number, rootId: number): boolean => {
                    let current: number | null = childId;
                    let depth = 0;
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
                            someMovedIn = true;
                        }
                    } else if (wasKnown) {
                        toRemoveIDs.push(item.id);
                    }
                }

                if (toRemoveIDs.length > 0) {
                    const removeSet = new Set(toRemoveIDs);
                    setLibraryState('items', items => items.filter(i => !removeSet.has(i.id)));
                }

                if (someMovedIn) {
                    libraryActions.refreshAssets(false);
                }
            });
        }
    },

    addLocation: async () => {
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const { metadataActions } = await import('../metadata');
            const { tauriService } = await import('../../tauri/services');
            const { addLocation: dbAddLocation } = await import('../../../lib/db');

            const selectedPath = await open({
                directory: true,
                multiple: false,
                title: 'Select Folder to Add'
            });

            if (selectedPath && !Array.isArray(selectedPath)) {
                await dbAddLocation(selectedPath);
                await metadataActions.loadLocations();
                await tauriService.startIndexing({ path: selectedPath });
                await libraryActions.refreshAssets(true);
                return { success: true, path: selectedPath };
            }
            return { success: false };
        } catch (err) {
            console.error('Failed to add location:', err);
            return { success: false, error: err };
        }
    },

    removeLocation: async (locationId: number) => {
        try {
            const { metadataActions } = await import('../metadata');
            await invoke('remove_location', { locationId });

            await Promise.all([metadataActions.loadLocations(), metadataActions.loadStats()]);

            await libraryActions.refreshAssets(true);

            return { success: true };
        } catch (err) {
            console.error('Failed to remove location:', err);
            return { success: false, error: err };
        }
    },

    applyTagToAssets: async (
        assetIds: number[],
        tagId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        if (assetIds.length === 0) {
            return {
                success: false,
                error: { code: ErrorCode.VALIDATION_ERROR, message: 'No items provided' }
            };
        }

        try {
            const { metadataActions, metadataState } = await import('../metadata');
            await tagService.addTagsToAssetsBatch(assetIds, [tagId]);
            await metadataActions.loadStats();

            if (filterState.selectedTags.length > 0) {
                await libraryActions.refreshAssets(false);
            }

            const tagName = metadataState.tags.find(tag => tag.id === tagId)?.name || 'Tag';
            return {
                success: true,
                data: {
                    tagName,
                    count: assetIds.length
                }
            };
        } catch (error) {
            console.error('Failed to apply tags to assets:', error);
            return {
                success: false,
                error: { code: ErrorCode.IO_ERROR, message: 'Failed to apply tags' }
            };
        }
    },

    applyTagToSelection: async (
        tagId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        return libraryActions.applyTagToAssets(selectionState.selectedIds, tagId);
    },

    applyTagToTarget: async (
        tagId: number,
        targetAssetId: number
    ): Promise<ActionResult<{ tagName: string; count: number }>> => {
        let targetIds = [targetAssetId];
        if (selectionState.selectedIds.includes(targetAssetId)) {
            targetIds = [...selectionState.selectedIds];
        }
        return libraryActions.applyTagToAssets(targetIds, tagId);
    },

    removeTagFromSelection: async (tagId: number): Promise<ActionResult> => {
        const selectedIds = selectionState.selectedIds;
        if (selectedIds.length === 0) {
            return {
                success: false,
                error: { code: ErrorCode.VALIDATION_ERROR, message: 'No items selected' }
            };
        }

        try {
            const { metadataActions } = await import('../metadata');
            await Promise.all(selectedIds.map(id => tagService.removeTagFromAsset(id, tagId)));
            await metadataActions.loadStats();
            if (filterState.selectedTags.length > 0) {
                await libraryActions.refreshAssets(false);
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
