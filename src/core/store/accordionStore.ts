import { createStore } from 'solid-js/store';

const STORAGE_KEY_PREFIX = 'mundam_accordion_expanded_';

interface AccordionState {
    [key: string]: string[];
}

/**
 * Loads the initial state for a specific accordion key from localStorage.
 */
function loadInitialState(key: string): string[] {
    const saved = localStorage.getItem(`${STORAGE_KEY_PREFIX}${key}`);
    if (!saved) return [];
    try {
        const parsed = JSON.parse(saved);
        return Array.isArray(parsed) ? parsed : [];
    } catch {
        return [];
    }
}

const [accordionStoreState, setAccordionStoreState] = createStore<AccordionState>({});

export const accordionActions = {
    /**
     * Ensures an accordion state exists for the given key and returns its current expanded items.
     * If no state exists in localStorage, it will initialize with the defaultValue.
     */
    initializeAccordion: (key: string, defaultValue: string[] = []) => {
        if (!accordionStoreState[key]) {
            const saved = localStorage.getItem(`${STORAGE_KEY_PREFIX}${key}`);
            setAccordionStoreState(key, saved ? loadInitialState(key) : defaultValue);
        }
        return accordionStoreState[key];
    },

    /**
     * Sets the expanded state of an accordion and persists it to localStorage.
     */
    setExpandedItems: (key: string, items: string[]) => {
        setAccordionStoreState(key, items);
        localStorage.setItem(`${STORAGE_KEY_PREFIX}${key}`, JSON.stringify(items));
    },

    /**
     * Clears expansion state for a specific accordion.
     */
    clearAll: (key: string) => {
        setAccordionStoreState(key, []);
        localStorage.removeItem(`${STORAGE_KEY_PREFIX}${key}`);
    }
};

export { accordionStoreState };
