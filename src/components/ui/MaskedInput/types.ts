import { InputProps } from '../Input/types';

/**
 * Properties for the MaskedInput component.
 * Extends standard InputProps and adds masking capability.
 */
export interface MaskedInputProps extends Omit<InputProps, 'onInput'> {
    /**
     * The input mask pattern.
     * Use '9' for digits. Other characters are treated as literal separators.
     * @example "99/99/9999"
     */
    mask: string;

    /**
     * Callback function executed when the masked value changes.
     * @param value - The new masked value.
     */
    onInput?: (value: string) => void;
}
