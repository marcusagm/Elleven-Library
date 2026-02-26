import { createMemo } from 'solid-js';
import { selectionState, selectionActions, isItemSelected } from '../store/selectionStore';

const selectedCountMemo = createMemo(() => selectionState.selectedIds.length);

/**
 * Hook providing access to the current selection state and management utilities.
 *
 * @returns {Object} Selection state and actions.
 */
export const useSelection = () => {
    return {
        // State
        get selectedIds() {
            return selectionState.selectedIds;
        },
        /** Memoized count of selected items */
        selectedCount: selectedCountMemo,

        // Actions
        toggle: selectionActions.toggle,
        select: selectionActions.select,
        selectRange: selectionActions.selectRange,
        clear: selectionActions.clear,
        /** Slow version of isSelected (reactive to whole list) */
        isSelected: selectionActions.isSelected,
        /** Fast version of isSelected (O(1) reactive) */
        isItemSelected
    };
};
