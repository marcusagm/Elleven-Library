import { createMemo } from 'solid-js';
import { libraryState, libraryActions } from '../store/libraryStore';

const itemsCountMemo = createMemo(() => libraryState.items.length);

/**
 * Hook providing access to library state and actions.
 *
 * @returns {Object} Library state accessors and refinement methods.
 */
export const useLibrary = () => {
    return {
        // State
        get items() {
            return libraryState.items;
        },
        get isFetching() {
            return libraryState.isFetching;
        },
        get totalItems() {
            return libraryState.totalItems;
        },
        /** Memoized count of items currently loaded in memory */
        loadedCount: itemsCountMemo,

        // Actions
        refreshImages: libraryActions.refreshImages,
        loadMore: libraryActions.loadMore,
        updateItemRating: libraryActions.updateItemRating,
        updateItemNotes: libraryActions.updateItemNotes,
        updateThumbnail: libraryActions.updateThumbnail,
        setThumbnailPriority: libraryActions.setThumbnailPriority
    };
};
