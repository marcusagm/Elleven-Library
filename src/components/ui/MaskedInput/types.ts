import { InputProps } from '../Input/types';

/**
 * Properties for the MaskedInput component.
 * Extends standard InputProps and adds masking capability.
 */
export interface MaskedInputProps extends Omit<InputProps, 'onInput'> {
    /**
     * The input mask pattern.
     * Tokens:
     * - '0': Numeric digits [0-9]
     * - 'a': Alpha characters [a-zA-Z]
     * - '*': Alphanumeric characters [a-zA-Z0-9]
     * Other characters are treated as literal separators.
     * @example "00/00/0000" (Date)
     * @example "aaa-0000" (License Plate)
     */
    mask: string;

    /**
     * Callback function executed when the masked value changes.
     * @param value - The new masked value.
     */
    onInput?: (value: string) => void;
}
