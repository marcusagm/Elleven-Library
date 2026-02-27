import { createStore } from 'solid-js/store';

const STORAGE_KEY_PREFIX = 'mundam_tree_expanded_';

interface TreeState {
    [key: string]: Set<string | number>;
}

/**
 * Loads the initial state for a specific tree key from localStorage.
 */
function loadInitialState(key: string): Set<string | number> {
    const saved = localStorage.getItem(`${STORAGE_KEY_PREFIX}${key}`);
    if (!saved) return new Set();
    try {
        const parsed = JSON.parse(saved);
        return Array.isArray(parsed) ? new Set(parsed) : new Set();
    } catch {
        return new Set();
    }
}

const [treeStoreState, setTreeStoreState] = createStore<TreeState>({});

export const treeActions = {
    /**
     * Ensures a tree state exists for the given key and returns its current IDs.
     */
    initializeTree: (key: string) => {
        if (!treeStoreState[key]) {
            setTreeStoreState(key, loadInitialState(key));
        }
        return treeStoreState[key];
    },

    /**
     * Toggles the expansion state of a node in a specific tree.
     */
    toggleExpansion: (key: string, id: string | number) => {
        const currentSet = treeActions.initializeTree(key);
        const nextSet = new Set(currentSet);
        if (nextSet.has(id)) {
            nextSet.delete(id);
        } else {
            nextSet.add(id);
        }
        setTreeStoreState(key, nextSet);
        localStorage.setItem(`${STORAGE_KEY_PREFIX}${key}`, JSON.stringify(Array.from(nextSet)));
    },

    /**
     * Forces an expansion state for a node in a specific tree.
     */
    setExpanded: (key: string, id: string | number, isExpanded: boolean) => {
        const currentSet = treeActions.initializeTree(key);
        const nextSet = new Set(currentSet);
        if (isExpanded) {
            nextSet.add(id);
        } else {
            nextSet.delete(id);
        }
        setTreeStoreState(key, nextSet);
        localStorage.setItem(`${STORAGE_KEY_PREFIX}${key}`, JSON.stringify(Array.from(nextSet)));
    },

    /**
     * Clears expansion state for a specific tree.
     */
    clearAll: (key: string) => {
        setTreeStoreState(key, new Set());
        localStorage.removeItem(`${STORAGE_KEY_PREFIX}${key}`);
    }
};

export { treeStoreState };
