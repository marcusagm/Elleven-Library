import { JSX } from 'solid-js';

/**
 * Available sizes for the Checkbox component.
 */
export type CheckboxSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the Checkbox component, extending standard HTML input attributes.
 */
export interface CheckboxProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'onChange' | 'type'
> {
    /**
     * The checked state of the checkbox.
     * @default false
     */
    checked?: boolean;
    /**
     * The default checked state of the checkbox when not explicitly controlled.
     * @default false
     */
    defaultChecked?: boolean;
    /**
     * Whether the checkbox should be in an indeterminate state (partial selection).
     * @default false
     */
    indeterminate?: boolean;
    /**
     * Callback function invoked when the checked state changes.
     * @param checked - The new checked state.
     */
    onCheckedChange?: (checked: boolean) => void;
    /**
     * The label text to display next to the checkbox.
     */
    label?: string;
    /**
     * A description to display below the label for additional context.
     */
    description?: string;
    /**
     * The size variant of the checkbox.
     * @default 'md'
     */
    size?: CheckboxSize;
}
