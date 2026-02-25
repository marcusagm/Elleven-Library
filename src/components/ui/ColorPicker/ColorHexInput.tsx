import { Component } from 'solid-js';
import { useColorPickerContext } from './context';

/**
 * Hexadecimal input field for precisely entering color codes.
 * Part of the ColorPicker compound component.
 *
 * Note: This component expects to find the input state in the context.
 * We should probably expose the input signals in the context.
 */
export const ColorHexInput: Component = () => {
    // We need to update the context to include the input signals
    // Since I'm creating the files now, I'll assume the context has them.
    // I need to go back and update ColorPickerContextValue and ColorPicker.tsx

    // Actually, I can just use a local signal if I want, but it's better to share it.
    // I'll update the context in the next step.

    const context = useColorPickerContext();

    return (
        <input
            type="text"
            class="ui-color-picker-input"
            value={context.activeHexadecimalInput()}
            onInput={event => {
                const newValue = event.currentTarget.value;
                context.setHexadecimalInput(newValue);
                context.setColor(newValue);
            }}
            maxLength={11}
            spellcheck={false}
            aria-label="Hexadecimal color value"
        />
    );
};
