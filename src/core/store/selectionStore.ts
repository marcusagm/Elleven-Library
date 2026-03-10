import { createStore } from 'solid-js/store';
import { createSelector } from 'solid-js';

interface SelectionState {
    /** Array of currently selected item IDs */
    selectedIds: string[];
    /** The ID of the last item interacted with, used as an anchor for range selection */
    lastSelectedId: string | null;
}

const [selectionState, setSelectionState] = createStore<SelectionState>({
    selectedIds: [],
    lastSelectedId: null
});

export const selectionActions = {
    /**
     * Toggles the selection of a specific item.
     * @param id - The ID of the item to toggle.
     * @param multi - If true, keeps existing selection (CMD/CTRL key).
     */
    toggle: (id: string, multi: boolean) => {
        if (multi) {
            const current = selectionState.selectedIds;
            if (current.includes(id)) {
                setSelectionState(
                    'selectedIds',
                    current.filter((i: string) => i !== id)
                );
                setSelectionState('lastSelectedId', null);
            } else {
                setSelectionState('selectedIds', [...current, id]);
                setSelectionState('lastSelectedId', id);
            }
        } else {
            setSelectionState('selectedIds', [id]);
            setSelectionState('lastSelectedId', id);
        }
    },

    /**
     * Selects a range of items between the last selected item and the current one.
     * @param id - The ID of the item clicked with SHIFT.
     * @param itemIds - Ordered list of all item IDs in the current view.
     */
    selectRange: (id: string, itemIds: string[]) => {
        const lastId = selectionState.lastSelectedId;
        if (lastId === null || lastId === id) {
            selectionActions.toggle(id, true);
            return;
        }

        const startIndex = itemIds.indexOf(lastId);
        const endIndex = itemIds.indexOf(id);

        if (startIndex === -1 || endIndex === -1) {
            selectionActions.toggle(id, true);
            return;
        }

        const [min, max] = startIndex < endIndex ? [startIndex, endIndex] : [endIndex, startIndex];
        const rangeIds = itemIds.slice(min, max + 1);

        // Merge with current selection
        const currentSet = new Set(selectionState.selectedIds);
        rangeIds.forEach(rangeId => currentSet.add(rangeId));

        setSelectionState('selectedIds', Array.from(currentSet));
        setSelectionState('lastSelectedId', id);
    },

    /**
     * Replaces the current selection with a new set of IDs.
     * @param ids - The new array of selected IDs.
     */
    select: (ids: string[]) => {
        setSelectionState('selectedIds', ids);
        setSelectionState('lastSelectedId', ids.length > 0 ? ids[ids.length - 1] : null);
    },

    /**
     * Clears the entire selection.
     */
    clear: () => {
        setSelectionState('selectedIds', []);
        setSelectionState('lastSelectedId', null);
    },

    /**
     * Check if an item is selected.
     * Note: For high-performance UI (AssetCard), use specialized selectors or stores
     * to avoid full list iterations.
     */
    isSelected: (id: string) => {
        return selectionState.selectedIds.includes(id);
    }
};

/**
 * Performance-optimized selector for item selection state.
 * Use this in AssetCard to avoid massive re-renders when selection changes.
 * It only triggers updates for elements whose selection status actually changed.
 */
export const isItemSelected = createSelector(
    () => selectionState.selectedIds,
    (id: string, list: string[]) => list.includes(id)
);

export { selectionState };
