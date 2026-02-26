import { treeState, treeActions } from '../store/treeStore';

/**
 * Hook for accessing and managing persistent tree expansion state.
 */
export const useTree = () => {
    return {
        // State
        get expandedIds() {
            return treeState.expandedIds;
        },

        // Actions
        toggle: treeActions.toggleExpansion,
        setExpanded: treeActions.setExpanded,
        clear: treeActions.clearAll
    };
};
