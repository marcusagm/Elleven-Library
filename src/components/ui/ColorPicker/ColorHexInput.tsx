import { Component } from 'solid-js';
import { useColorPickerContext } from './context';

/**
 * Hexadecimal input field for precisely entering color codes.
 * Part of the ColorPicker compound component.
 *
 * @returns {import('solid-js').JSX.Element} The rendered input field.
 */
export const ColorHexInput: Component = () => {
    const colorPickerContext = useColorPickerContext();

    return (
        <input
            type="text"
            class="ui-color-picker-input"
            value={colorPickerContext.activeHexadecimalInput()}
            onInput={event => {
                const newValue = event.currentTarget.value;
                colorPickerContext.setHexadecimalInput(newValue);
                colorPickerContext.setColor(newValue);
            }}
            maxLength={11}
            spellcheck={false}
            aria-label="Hexadecimal color value"
        />
    );
};
