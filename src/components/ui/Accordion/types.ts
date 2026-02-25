import { JSX, Accessor } from 'solid-js';

/**
 * Defines how much of the accordion can be open at once.
 * - 'single': Only one item can be expanded at a time.
 * - 'multiple': Multiple items can be expanded simultaneously.
 */
export type AccordionType = 'single' | 'multiple';

/**
 * Properties for the root Accordion component.
 */
export interface AccordionRootProps {
    /** Whether the accordion supports single or multiple expanded items. Defaults to 'single'. */
    type?: AccordionType;
    /** Controlled array of expanded item values. */
    value?: string[];
    /** Initial array of expanded item values (uncontrolled). Defaults to []. */
    defaultValue?: string[];
    /** Callback triggered when the expanded items set changes. */
    onValueChange?: (value: string[]) => void;
    /** For 'single' type: whether the active item can be collapsed. Defaults to true. */
    collapsible?: boolean;
    /** Whether the entire accordion is disabled. */
    disabled?: boolean;
    /** Optional CSS class for the root element. */
    class?: string;
    /** Children elements, typically AccordionItem. */
    children?: JSX.Element;
}

/**
 * Properties for individual Accordion items.
 */
export interface AccordionItemProps {
    /** Unique identifier for this item. */
    value: string;
    /** Whether this item is disabled. */
    disabled?: boolean;
    /** Optional CSS class for the item container. */
    class?: string;
    /** Children elements, typically AccordionTrigger and AccordionContent. */
    children?: JSX.Element;
}

/**
 * Properties for the Accordion trigger (the header).
 */
export interface AccordionTriggerProps {
    /** Optional CSS class for the trigger button. */
    class?: string;
    /** Children elements to render inside the trigger button. */
    children?: JSX.Element;
}

/**
 * Properties for the Accordion content (the expandable section).
 */
export interface AccordionContentProps {
    /** Optional CSS class for the content container. */
    class?: string;
    /** Children elements to render inside the content section. */
    children?: JSX.Element;
}

/**
 * Internal context for the Accordion to share state between the Root and sub-components.
 */
export interface AccordionContextValue {
    /** Accessor for the array of currently expanded item values. */
    expandedItems: Accessor<string[]>;
    /** Toggles the expansion state of a specific item. */
    toggleItem: (itemValue: string) => void;
    /** Whether the entire accordion is disabled. */
    disabled: Accessor<boolean>;
    /** The type of accordion behavior. */
    type: Accessor<AccordionType>;
}

/**
 * Internal context for an individual Accordion item.
 */
export interface AccordionItemContextValue {
    /** The unique value of the current item. */
    value: Accessor<string>;
    /** Whether this specific item is disabled. */
    disabled: Accessor<boolean>;
    /** Whether this specific item is currently expanded. */
    isExpanded: Accessor<boolean>;
    /** ID for the trigger element, used for ARIA relationships. */
    triggerId: string;
    /** ID for the content element, used for ARIA relationships. */
    contentId: string;
}
