import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { ToggleProperties } from './types';
import './toggle.css';

/**
 * Toggle button for binary on/off states with visual feedback.
 * Unlike a Switch, Toggle is a button that shows a pressed state visually through its style.
 *
 * @param {ToggleProperties} properties - Properties for the Toggle component.
 * @returns {JSX.Element} The rendered toggle button.
 *
 * @example
 * <Toggle aria-label="Toggle bold">
 *   <Bold size={16} />
 * </Toggle>
 *
 * @example
 * <Toggle variant="outline" size="sm" pressed={isBold()} onPressedChange={setIsBold}>
 *   B
 * </Toggle>
 */
export const Toggle: Component<ToggleProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'pressed',
        'defaultPressed',
        'onPressedChange',
        'variant',
        'size',
        'disabled',
        'children'
    ]);

    const { value: isPressed, setValue: setPressed } = createControllableSignal({
        value: () => localProperties.pressed,
        defaultValue: localProperties.defaultPressed ?? false,
        onChange: (pressed: boolean) => localProperties.onPressedChange?.(pressed)
    });

    /**
     * Toggles the pressed state when the button is clicked.
     */
    const handleToggleClick = () => {
        if (localProperties.disabled) return;
        setPressed(!isPressed());
    };

    const activeVariant = () => localProperties.variant || 'default';
    const activeSize = () => localProperties.size || 'md';

    return (
        <button
            type="button"
            class={cn(
                'ui-toggle',
                `ui-toggle-${activeVariant()}`,
                `ui-toggle-${activeSize()}`,
                isPressed() && 'ui-toggle-pressed',
                localProperties.class
            )}
            aria-pressed={isPressed()}
            disabled={localProperties.disabled}
            onClick={handleToggleClick}
            {...remainingProperties}
        >
            {localProperties.children}
        </button>
    );
};
