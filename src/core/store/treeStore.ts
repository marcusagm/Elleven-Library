import { createStore } from 'solid-js/store';
import { createEffect } from 'solid-js';

const STORAGE_KEY = 'mundam_folder_expanded';

interface TreeState {
    expandedIds: Set<string | number>;
}

/**
 * Persists the tree expansion state across sessions.
 * Currently uses localStorage but could be migrated to a database or config file.
 */
function loadInitialState(): Set<string | number> {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (!saved) return new Set();
    try {
        const parsed = JSON.parse(saved);
        return Array.isArray(parsed) ? new Set(parsed) : new Set();
    } catch {
        return new Set();
    }
}

const [treeState, setTreeState] = createStore<TreeState>({
    expandedIds: loadInitialState()
});

/**
 * Persist state on change.
 */
createEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(Array.from(treeState.expandedIds)));
});

export const treeActions = {
    /**
     * Toggles the expansion state of a specific node.
     * @param id - The unique identifier of the tree node.
     */
    toggleExpansion: (id: string | number) => {
        const nextSet = new Set(treeState.expandedIds);
        if (nextSet.has(id)) {
            nextSet.delete(id);
        } else {
            nextSet.add(id);
        }
        setTreeState('expandedIds', nextSet);
    },

    /**
     * Forces an expansion state for a node.
     * @param id - The node identifier.
     * @param isExpanded - Whether the node should be expanded.
     */
    setExpanded: (id: string | number, isExpanded: boolean) => {
        const nextSet = new Set(treeState.expandedIds);
        if (isExpanded) {
            nextSet.add(id);
        } else {
            nextSet.delete(id);
        }
        setTreeState('expandedIds', nextSet);
    },

    /**
     * Clears all expansion state.
     */
    clearAll: () => {
        setTreeState('expandedIds', new Set());
    }
};

export { treeState };
