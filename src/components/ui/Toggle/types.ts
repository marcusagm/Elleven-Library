import { JSX, Accessor } from 'solid-js';

/**
 * Visual style variants for the Toggle component.
 */
export type ToggleVariant = 'default' | 'outline';

/**
 * Size variants for the Toggle component.
 */
export type ToggleSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the Toggle component.
 */
export interface ToggleProperties extends Omit<
    JSX.ButtonHTMLAttributes<HTMLButtonElement>,
    'onChange'
> {
    /** Whether the toggle is currently pressed (controlled). */
    pressed?: boolean;
    /** Whether the toggle is pressed by default (uncontrolled). */
    defaultPressed?: boolean;
    /** Callback invoked when the pressed state changes. */
    onPressedChange?: (pressed: boolean) => void;
    /** Visual style variant. @default 'default' */
    variant?: ToggleVariant;
    /** Size variant. @default 'md' */
    size?: ToggleSize;
    /** Content of the toggle. */
    children: JSX.Element;
}

/**
 * Types of selection behavior for ToggleGroup.
 */
export type ToggleGroupType = 'single' | 'multiple';

/**
 * Size variants for ToggleGroup and its items.
 */
export type ToggleGroupSize = 'sm' | 'md' | 'lg' | 'xl';

/**
 * Properties for a ToggleGroup with single selection mode.
 */
export interface ToggleGroupSingleProperties {
    type: 'single';
    /** Current value (controlled). */
    value?: string;
    /** Default value (uncontrolled). */
    defaultValue?: string;
    /** Callback invoked when value changes. */
    onValueChange?: (value: string) => void;
}

/**
 * Properties for a ToggleGroup with multiple selection mode.
 */
export interface ToggleGroupMultipleProperties {
    type: 'multiple';
    /** Current values (controlled). */
    value?: string[];
    /** Default values (uncontrolled). */
    defaultValue?: string[];
    /** Callback invoked when values change. */
    onValueChange?: (value: string[]) => void;
}

/**
 * Union type for ToggleGroup properties.
 */
export type ToggleGroupProperties = (
    | ToggleGroupSingleProperties
    | ToggleGroupMultipleProperties
) & {
    /** Whether the entire group is disabled. */
    disabled?: boolean;
    /** Layout orientation. @default 'horizontal' */
    orientation?: 'horizontal' | 'vertical';
    /** Size variant for items in the group. @default 'md' */
    size?: ToggleGroupSize;
    /** Custom CSS class. */
    class?: string;
    /** Items of the group. */
    children: JSX.Element;
};

/**
 * Value provided by the ToggleGroup context to its items.
 */
export interface ToggleGroupContextValue {
    /** Selection mode. */
    type: ToggleGroupType;
    /** Current selection(s) accessor. */
    value: Accessor<string | string[]>;
    /** Callback for item clicks. */
    onItemClick: (itemValue: string) => void;
    /** Group disabled state. */
    disabled: boolean;
    /** Item size accessor. */
    size: Accessor<ToggleGroupSize>;
}

/**
 * Properties for individual items within a ToggleGroup.
 */
export interface ToggleGroupItemProperties extends Omit<
    JSX.ButtonHTMLAttributes<HTMLButtonElement>,
    'value'
> {
    /** Unique value for this item. */
    value: string;
    /** Content of the item. */
    children: JSX.Element;
}
