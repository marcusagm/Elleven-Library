import { Component, splitProps } from 'solid-js';
import { Input } from '../Input/Input';
import { MaskedInputProps } from './types';

/**
 * A specialized Input component that applies a format mask to the user input.
 * Supports '9' as a placeholder for numeric digits.
 *
 * @param props - Properties for the MaskedInput component.
 * @returns The rendered MaskedInput component.
 *
 * @example
 * <MaskedInput mask="99/99/9999" placeholder="DD/MM/YYYY" onInput={setDate} />
 */
export const MaskedInput: Component<MaskedInputProps> = props => {
    // Separate specialized masking logic properties from standard input attributes.
    // We avoid abbreviations to comply with naming guidelines.
    const [maskComponentProperties, remainingHtmlAttributes] = splitProps(props, [
        'mask',
        'onInput',
        'value',
        'error',
        'errorMessage'
    ]);

    /**
     * Applies the defined mask pattern to a raw string value.
     *
     * @param rawValue - The unmasked input value.
     * @returns The formatted string based on the mask.
     */
    const applyInputMask = (rawValue: string) => {
        let resultString = '';
        let valueIndex = 0;
        // Remove all non-digit characters before applying the mask.
        const cleanedValue = rawValue.replace(/\D/g, '');

        for (
            let maskIndex = 0;
            maskIndex < maskComponentProperties.mask.length && valueIndex < cleanedValue.length;
            maskIndex++
        ) {
            const currentMaskCharacter = maskComponentProperties.mask[maskIndex];

            if (currentMaskCharacter === '9') {
                resultString += cleanedValue[valueIndex];
                valueIndex++;
            } else {
                resultString += currentMaskCharacter;
            }
        }

        return resultString;
    };

    /**
     * Handles the input event, applying the mask and updating the element's value.
     *
     * @param event - The input event from the HTML input element.
     */
    const handleMaskedInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const maskedValue = applyInputMask(event.currentTarget.value);

        // Directly update the input element's value to reflect the mask immediately.
        event.currentTarget.value = maskedValue;

        // Execute the optional onInput callback with the formatted value.
        maskComponentProperties.onInput?.(maskedValue);
    };

    return (
        <Input
            {...remainingHtmlAttributes}
            value={maskComponentProperties.value}
            error={maskComponentProperties.error}
            errorMessage={maskComponentProperties.errorMessage}
            onInput={handleMaskedInput}
        />
    );
};
