import { Component, splitProps, mergeProps, createSignal, createEffect, JSX } from 'solid-js';
import { ColorInputProps } from './types';
import { ColorPicker } from '../ColorPicker';
import { Popover } from '../Popover';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { validateHexadecimalColor, normalizeHexadecimalValue } from '../ColorPicker/utils';
import './color-input.css';

/**
 * Specialized Input component for color selection.
 * Combines a text field for hexadecimal entry with a Popover containing a ColorPicker.
 *
 * @param {ColorInputProps} props - Component properties.
 * @returns {JSX.Element} The rendered component.
 *
 * @example
 * <ColorInput
 *   label="Accent Color"
 *   value={color()}
 *   onChange={setColor}
 * />
 */
export const ColorInput: Component<ColorInputProps> = properties => {
    const merged = mergeProps({ defaultValue: '#000000', size: 'md' as const }, properties);
    const [local, others] = splitProps(merged, [
        'value',
        'defaultValue',
        'onChange',
        'class',
        'wrapperClass',
        'label',
        'error',
        'errorMessage',
        'disabled',
        'size'
    ]);

    // Managed color state
    const { value: activeColor, setValue: setActiveColor } = createControllableSignal({
        value: () => local.value,
        defaultValue: local.defaultValue,
        onChange: (color: string) => local.onChange?.(color)
    });

    // Internal signal for the text field to allow typing partial hex codes
    const [hexadecimalInputValue, setHexadecimalInputValue] = createSignal(activeColor() || '');

    // Sync external color changes to the input field
    createEffect(() => {
        const currentColor = activeColor();
        if (currentColor) {
            setHexadecimalInputValue(currentColor);
        }
    });

    /**
     * Handles live typing in the hexadecimal field.
     */
    const handleInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const newValue = event.currentTarget.value;
        setHexadecimalInputValue(newValue);

        // Update the actual color value immediately if valid
        if (validateHexadecimalColor(newValue)) {
            setActiveColor(newValue);
        }
    };

    /**
     * Normalizes the color on blur (e.g., adds '#' or expands shorthand).
     */
    const handleBlur = () => {
        const normalized = normalizeHexadecimalValue(hexadecimalInputValue());

        if (normalized) {
            setActiveColor(normalized);
            setHexadecimalInputValue(normalized);
        } else {
            // Revert to last valid color if invalid
            setHexadecimalInputValue(activeColor() || '#000000');
        }
    };

    /**
     * Callback for color changes coming from the ColorPicker popover.
     */
    const handlePickerChange = (color: string) => {
        setActiveColor(color);
        setHexadecimalInputValue(color);
    };

    /**
     * Render the color swatch button that acts as the popover trigger.
     *
     * @returns {JSX.Element} The rendered swatch trigger button.
     */
    const renderSwatchButton = (): JSX.Element => (
        <button
            type="button"
            class="ui-color-input-swatch-btn"
            disabled={local.disabled}
            aria-label="Open color picker"
        >
            <div
                class="ui-color-input-swatch"
                style={{
                    'background-color': activeColor(),
                    'background-image':
                        activeColor() === 'transparent'
                            ? 'linear-gradient(45deg, #ccc 25%, transparent 25%), linear-gradient(-45deg, #ccc 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #ccc 75%), linear-gradient(-45deg, transparent 75%, #ccc 75%)'
                            : 'none',
                    'background-size': '8px 8px',
                    'background-position': '0 0, 0 4px, 4px -4px, -4px 0px'
                }}
            />
        </button>
    );

    return (
        <div class={cn('ui-color-input-wrapper', local.wrapperClass)}>
            {local.label && <label class="ui-color-input-label">{local.label}</label>}
            <div
                class={cn(
                    'ui-color-input-container',
                    `ui-color-input-${local.size}`,
                    local.error && 'ui-color-input-error',
                    local.disabled && 'ui-color-input-disabled',
                    local.class
                )}
            >
                <div class="ui-color-input-icon-left">
                    {local.disabled ? (
                        renderSwatchButton()
                    ) : (
                        <Popover trigger={renderSwatchButton()} class="ui-color-input-popover">
                            <ColorPicker
                                color={activeColor() || '#000000'}
                                onChange={handlePickerChange}
                                allowNoColor
                            />
                        </Popover>
                    )}
                </div>

                <input
                    class="ui-color-input-field"
                    value={hexadecimalInputValue()}
                    onInput={handleInput}
                    onBlur={handleBlur}
                    disabled={local.disabled}
                    maxLength={9} // #RRGGBBAA
                    aria-invalid={local.error || undefined}
                    {...others}
                />
            </div>

            {local.error && local.errorMessage && (
                <span class="ui-color-input-error-message" role="alert">
                    {local.errorMessage}
                </span>
            )}
        </div>
    );
};
