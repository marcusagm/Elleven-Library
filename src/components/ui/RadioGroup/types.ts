import { JSX } from 'solid-js';

/**
 * Valid orientations for the RadioGroup.
 */
export type RadioGroupOrientation = 'horizontal' | 'vertical';

/**
 * Size variants for radio items.
 */
export type RadioGroupSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the RadioGroup root component.
 */
export interface RadioGroupProperties {
    /** The currently selected value. */
    value?: string;
    /** The default selected value when not controlled. */
    defaultValue?: string;
    /** Callback for selection changes. */
    onValueChange?: (value: string) => void;
    /** Name attribute for the group. */
    name?: string;
    /** Whether the entire group is disabled. */
    disabled?: boolean;
    /** Visual orientation of items. */
    orientation?: RadioGroupOrientation;
    /** Custom CSS class. */
    class?: string;
    /** Children elements (RadioGroupItem). */
    children: JSX.Element;
}

/**
 * Properties for a RadioGroupItem.
 */
export interface RadioGroupItemProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'type' | 'onChange'
> {
    /** Unique value for this option. */
    value: string;
    /** Label text to display. */
    label?: string;
    /** Context description displayed below the label. */
    description?: string;
    /** Size variant of the radio indicator. */
    size?: RadioGroupSize;
}
