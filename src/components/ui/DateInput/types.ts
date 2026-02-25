import { JSX } from 'solid-js';

/**
 * Properties defining the behavior, look, and state of the DateInput component.
 * Extends standard HTML input attributes while specializing behavior for Date objects.
 */
export interface DateInputProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'value' | 'onChange' | 'onInput' | 'defaultValue'
> {
    /** The reactive Date object representing the current selection. Use null to clear the field. */
    value?: Date | null;

    /** The initial Date value to be used when the component is uncontrolled. */
    defaultValue?: Date | null;

    /**
     * Callback function executed when either a valid date is typed or selected from the picker.
     * @param date - The updated Date object or null if cleared.
     */
    onChange?: (date: Date | null) => void;

    /** An optional text label to be displayed above the input field for accessibility and context. */
    label?: string;

    /** Flag indicating if the input should visually reflect an invalid or error state. */
    error?: boolean;

    /** A descriptive message explaining the validation error, displayed below the input field. */
    errorMessage?: string;

    /** An optional CSS class to be applied to the outermost wrapper container of the component. */
    wrapperClass?: string;

    /**
     * The visual density and scale of the input field.
     * @default 'md'
     */
    size?: 'sm' | 'md' | 'lg';
}
