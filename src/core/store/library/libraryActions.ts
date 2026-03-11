import { reconcile } from 'solid-js/store';
import { untrack } from 'solid-js';
import { invokeCommand as invoke } from '../../../lib/api';

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
import type {
    SearchGroup as V2SearchGroup,
    SearchCriterion as V2SearchCriterion
} from '../../../types';
import type { SearchGroup as UISearchGroup } from '../filter/schemas';

const mapToV2SearchGroup = (group: UISearchGroup): V2SearchGroup => ({
    id: group.id,
    logicalOperator: group.logicalOperator,
    items: group.items.map(item => {
        if ('items' in item) {
            return mapToV2SearchGroup(item as UISearchGroup);
        }
        return {
            id: item.id,
            key: item.key,
            operator: item.operator,
            value: item.value
        } as V2SearchCriterion;
    })
});

export const libraryActions = {
    ...itemActions,

    fetchLibraryBatch: async (offset: number) => {
        const anyFilter = filterActions.hasActiveFilters();

        let filterParams: import('../../../types').AssetFilter = {};

        if (anyFilter) {
            filterParams = {
                untagged: filterState.filterUntagged ? true : undefined,
                folderId: filterState.selectedFolderId?.toString() || undefined,
                tags: filterState.selectedTags.map(String),
                searchQuery: filterState.searchQuery
            };

            if (filterState.advancedSearch) {
                return await tagService.searchAssets(
                    { id: 'v2-search', rootGroup: mapToV2SearchGroup(filterState.advancedSearch) },
                    offset / BATCH_SIZE + 1, // converting offset to page
                    BATCH_SIZE
                );
            }
        }

        return await tagService.getAssets(filterParams, offset / BATCH_SIZE + 1, BATCH_SIZE);
    },

    refreshTotalCount: () => {
        // V2 search_assets / get_assets endpoints already return totalItems via PageInfo.
        // We might want to remove this distinct count call, or map it properly.
        // For now, retaining a mock compatibility using searchAssets assuming it returns a wrapper or we adjust it.
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
                const toRemoveIDs: string[] = [];

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

    removeLocation: async (locationId: number | string) => {
        try {
            const { metadataActions } = await import('../metadata');
            await invoke('remove_location', { folderId: String(locationId) });

            await Promise.all([metadataActions.loadLocations(), metadataActions.loadStats()]);

            await libraryActions.refreshAssets(true);

            return { success: true };
        } catch (err) {
            console.error('Failed to remove location:', err);
            return { success: false, error: err };
        }
    },

    applyTagToAssets: async (
        assetIds: string[],
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

            // Reusing updateAssetsTags which properly triggers notifications and emits events
            await metadataActions.updateAssetsTags(assetIds, [tagId], 'merge');

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
        targetAssetId: string
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
            // Reusing updateAssetsTags to properly emit events and bump tagUpdateVersion
            await metadataActions.updateAssetsTags(selectedIds, [tagId], 'remove');
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
