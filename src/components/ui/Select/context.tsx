import { Accessor, Setter, createContext, useContext } from 'solid-js';
import { SelectOption } from './types';

/**
 * Shared state for Select compound components.
 */
export interface SelectContextValue {
    /** The actual selected value (reactive). */
    value: Accessor<string>;
    /** Updates the selected value. */
    setValue: (value: string) => void;
    /** Whether the dropdown menu is visible. */
    isOpen: Accessor<boolean>;
    /** Shows or hides the dropdown menu. */
    setIsOpen: Setter<boolean>;
    /** Whether the entire component is disabled. */
    disabled: Accessor<boolean | undefined>;
    /** The current search filter query. */
    searchQuery: Accessor<string>;
    /** Updates the current search filter query. */
    setSearchQuery: Setter<string>;
    /** The index of the currently highlighted option. */
    highlightedIndex: Accessor<number>;
    /** Updates the highlighted option index. */
    setHighlightedIndex: Setter<number>;
    /** Current coordinates for portal contents. */
    contentPosition: Accessor<{ top: number; left: number; width: number }>;
    /** Updates the coordinates for portal contents. */
    setContentPosition: Setter<{ top: number; left: number; width: number }>;
    /** Reference to the trigger element for positioning. */
    triggerElement: Accessor<HTMLButtonElement | undefined>;
    /** Sets the reference to the trigger element. */
    setTriggerElement: (element: HTMLButtonElement) => void;
    /** Reference to the content element. */
    contentElement: Accessor<HTMLDivElement | undefined>;
    /** Sets the reference to the content element. */
    setContentElement: (element: HTMLDivElement) => void;
    /** Current list of all available options data. */
    options: Accessor<SelectOption[]>;
}

/**
 * Context for managing the internal state of Select component suite.
 */
export const SelectContext = createContext<SelectContextValue>();

/**
 * Hook to consume Select context within child components.
 * @throws {Error} If used outside a Select.Root provider.
 */
export const useSelect = () => {
    const context = useContext(SelectContext);
    if (!context) {
        throw new Error('Select components must be used within a Select.Root');
    }
    return context;
};
