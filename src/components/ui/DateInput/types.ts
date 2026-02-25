import { JSX } from 'solid-js';

/**
 * Properties for the DateInput component.
 * Extends standard input HTML attributes but overrides value and event handlers
 * to work with Date objects instead of raw strings.
 */
export interface DateInputProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'value' | 'onChange' | 'onInput' | 'defaultValue'
> {
    /** The currently selected date value. */
    value?: Date | null;

    /** The initial date value for uncontrolled usage. */
    defaultValue?: Date | null;

    /** Callback triggered when a valid date is selected or entered. */
    onChange?: (date: Date | null) => void;

    /** Optional text label displayed above the input field. */
    label?: string;

    /** Whether the input is in an error state. */
    error?: boolean;

    /** Message to display when the input is in an error state. */
    errorMessage?: string;

    /** Additional CSS class for the outermost wrapper element. */
    wrapperClass?: string;

    /**
     * The size of the input field.
     * @default 'md'
     */
    size?: 'sm' | 'md' | 'lg';
}
