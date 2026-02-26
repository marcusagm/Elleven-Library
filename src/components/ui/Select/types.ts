import { JSX } from 'solid-js';
import { SelectContextValue } from './context';

/**
 * Valid sizes for the Select component.
 */
export type SelectSize = 'sm' | 'md' | 'lg';

/**
 * Represents a single option within the Select component.
 */
export interface SelectOption {
    /** Unique value for the option. */
    value: string;
    /** User-friendly label text. */
    label: string;
    /** Whether the option is selectable. */
    disabled?: boolean;
}

/**
 * Properties for the Select root component.
 */
export interface SelectRootProperties {
    /** List of options to choose from (used mainly by the high-level Select). */
    options?: SelectOption[];
    /** The current selected value (controlled). */
    value?: string;
    /** The initial selected value (uncontrolled). */
    defaultValue?: string;
    /** Callback triggered when selection changes. */
    onValueChange?: (value: string) => void;
    /** Whether the select is interactive. */
    disabled?: boolean;
    /** Identifier for form submission. */
    name?: string;
    /** Children elements or a render function that receives the Select context. */
    children?: JSX.Element | ((context: SelectContextValue) => JSX.Element);
    /** Reference identifier. */
    id?: string;
}

/**
 * Properties for the Select trigger component.
 */
export interface SelectTriggerProperties extends JSX.HTMLAttributes<HTMLButtonElement> {
    /** Error state for the trigger. */
    error?: boolean;
    /** Size variant of the trigger. */
    size?: SelectSize;
}

/**
 * Properties for the Select item component.
 */
export interface SelectItemProperties {
    /** The option data represented by this item. */
    option: SelectOption;
    /** Custom CSS class. */
    class?: string;
}

/**
 * Properties for the Select content (dropdown) component.
 */
export interface SelectContentProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Additional CSS class. */
    class?: string;
}

/**
 * Properties for the high-level Select component (Backward compatibility & Convenience).
 */
export interface SelectProperties extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
    /** List of all options. */
    options: SelectOption[];
    /** Selected value. */
    value?: string;
    /** Default value. */
    defaultValue?: string;
    /** Change callback. */
    onValueChange?: (value: string) => void;
    /** Text to show when no value is selected. */
    placeholder?: string;
    /** Disabled state. */
    disabled?: boolean;
    /** Shows a clear button. */
    clearable?: boolean;
    /** Enables search filtering. */
    searchable?: boolean;
    /** Form name. */
    name?: string;
    /** Error status. */
    error?: boolean;
    /** Error message text. */
    errorMessage?: string;
    /** UI icon for the left side. */
    leftIcon?: JSX.Element;
    /** UI icon for the right side (before chevron). */
    rightIcon?: JSX.Element;
    /** Size variant. */
    size?: SelectSize;
}
