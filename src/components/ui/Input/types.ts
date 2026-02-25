import { JSX } from 'solid-js';

/**
 * Valid size variants for the Input component.
 */
export type InputSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the Input component, extending standard HTML input attributes.
 */
export interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
    /**
     * Label text to display above the input field.
     */
    label?: string;

    /**
     * Optional icon element to display on the left side of the input.
     */
    leftIcon?: JSX.Element;

    /**
     * Optional icon element to display on the right side of the input.
     */
    rightIcon?: JSX.Element;

    /**
     * The size variant of the input.
     * @default 'md'
     */
    size?: InputSize;

    /**
     * Whether the input is in an error state.
     */
    error?: boolean;

    /**
     * Error message to display below the input when the error state is active.
     */
    errorMessage?: string;

    /**
     * Additional CSS class name for the outermost wrapper element.
     */
    wrapperClass?: string;

    /**
     * Additional CSS class name for the input element itself.
     */
    class?: string;
}
