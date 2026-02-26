import { InputProps } from '../Input/types';

/**
 * Properties for the NumberInput component.
 * Extends standard InputProps but manages numeric values and validation.
 */
export interface NumberInputProps extends Omit<
    InputProps,
    'onChange' | 'onInput' | 'value' | 'defaultValue'
> {
    /**
     * The numeric value of the input.
     */
    value?: number;

    /**
     * The initial value when used in an uncontrolled manner.
     */
    defaultValue?: number;

    /**
     * Minimum allowed numeric value.
     */
    min?: number;

    /**
     * Maximum allowed numeric value.
     */
    max?: number;

    /**
     * The step value for incrementing and decrementing.
     * @default 1
     */
    step?: number;

    /**
     * Callback function executed when the numeric value changes.
     * @param value - The new numeric value or undefined if the input is empty.
     */
    onChange?: (value: number | undefined) => void;

    /**
     * Optional formatter function to display the value in a specific format.
     * @param value - The numeric value to format.
     * @returns A formatted string.
     */
    format?: (value: number) => string;
}
