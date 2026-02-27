import { treeStoreState, treeActions } from '../store/treeStore';

/**
 * Hook for accessing and managing persistent tree expansion state for a specific tree.
 *
 * @param key - Unique identifier for the tree (e.g., 'folders', 'tags').
 */
export const useTree = (key: string) => {
    // Ensure the tree is initialized in the store
    treeActions.initializeTree(key);

    return {
        // State
        get expandedIds() {
            return treeStoreState[key] || new Set();
        },

        // Actions
        toggle: (id: string | number) => treeActions.toggleExpansion(key, id),
        setExpanded: (id: string | number, isExpanded: boolean) =>
            treeActions.setExpanded(key, id, isExpanded),
        clear: () => treeActions.clearAll(key)
    };
};
