import { InputProps } from '../Input';

/**
 * Properties for the ColorInput component.
 * Extends base InputProps while overriding color-specific fields.
 */
export interface ColorInputProps extends Omit<
    InputProps,
    'onChange' | 'onInput' | 'value' | 'defaultValue'
> {
    /** Current hexadecimal color value */
    value?: string;
    /** Default hexadecimal color value (defaults to '#000000') */
    defaultValue?: string;
    /** Callback triggered when the color is changed and validated */
    onChange?: (color: string) => void;
    /** Size variant of the input */
    size?: 'sm' | 'md' | 'lg';
}
