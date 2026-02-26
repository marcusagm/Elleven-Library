import { Component, splitProps } from 'solid-js';
import { Input } from '../Input/Input';
import { MaskedInputProps } from './types';

/**
 * Checks if a character matches a corresponding mask token.
 *
 * @param character - The character to check.
 * @param token - The mask token (0, a, *).
 * @returns True if the character matches the token rules.
 */
const matchesToken = (character: string, token: string): boolean => {
    if (token === '0') return /[0-9]/.test(character);
    if (token === 'a') return /[a-zA-Z]/.test(character);
    if (token === '*') return /[a-zA-Z0-9]/.test(character);
    return false;
};

/**
 * Determines if a character is a valid mask token.
 *
 * @param character - The character to evaluate.
 * @returns True if the character is a supported token.
 */
const isToken = (character: string): boolean => ['0', 'a', '*'].includes(character);

/**
 * Find the last index in rawValue that matches a token to avoid trailing separators.
 *
 * @param maskPattern - The mask string.
 * @param rawValue - The unmasked input value.
 * @returns The last valid index in rawValue.
 */
const calculateLastValidInputIndex = (maskPattern: string, rawValue: string): number => {
    let lastValidInputIndex = -1;
    let tempValueIndex = 0;

    for (let maskIndex = 0; maskIndex < maskPattern.length; maskIndex++) {
        const char = maskPattern[maskIndex];
        if (isToken(char)) {
            while (
                tempValueIndex < rawValue.length &&
                !matchesToken(rawValue[tempValueIndex], char)
            ) {
                tempValueIndex++;
            }
            if (tempValueIndex < rawValue.length) {
                lastValidInputIndex = tempValueIndex;
                tempValueIndex++;
            }
        } else if (tempValueIndex < rawValue.length && rawValue[tempValueIndex] === char) {
            tempValueIndex++;
        }
    }
    return lastValidInputIndex;
};

/**
 * A specialized Input component that applies a format mask to the user input.
 * Supports '0' (numeric), 'a' (alpha), and '*' (alphanumeric) as placeholders.
 *
 * @param props - Properties for the MaskedInput component.
 * @returns The rendered MaskedInput component.
 *
 * @example
 * <MaskedInput mask="00/00/0000" placeholder="DD/MM/YYYY" onInput={setDate} />
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
     * Supports tokens: 0 (digits), a (letters), * (alphanumeric).
     *
     * @param rawValue - The unmasked input value.
     * @returns The formatted string based on the mask.
     */
    const applyInputMask = (rawValue: string) => {
        if (!rawValue) return '';

        const maskPattern = maskComponentProperties.mask;
        const lastValidInputIndex = calculateLastValidInputIndex(maskPattern, rawValue);

        let formattedResult = '';
        let inputValueIndex = 0;

        for (let maskIndex = 0; maskIndex < maskPattern.length; maskIndex++) {
            const currentMaskChar = maskPattern[maskIndex];

            if (isToken(currentMaskChar)) {
                while (
                    inputValueIndex < rawValue.length &&
                    !matchesToken(rawValue[inputValueIndex], currentMaskChar)
                ) {
                    inputValueIndex++;
                }

                if (inputValueIndex < rawValue.length) {
                    formattedResult += rawValue[inputValueIndex];
                    inputValueIndex++;
                } else {
                    break;
                }
            } else {
                // Only add separator if we haven't reached the end of valid input
                if (inputValueIndex <= lastValidInputIndex) {
                    formattedResult += currentMaskChar;
                }

                if (
                    inputValueIndex < rawValue.length &&
                    rawValue[inputValueIndex] === currentMaskChar
                ) {
                    inputValueIndex++;
                }
            }
        }

        return formattedResult;
    };

    /**
     * Handles the input event, applying the mask and updating the element's value.
     *
     * @param event - The input event from the HTML input element.
     */
    const handleMaskedInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const inputElement = event.currentTarget;
        const maskedValue = applyInputMask(inputElement.value);

        // Update the value
        inputElement.value = maskedValue;

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
