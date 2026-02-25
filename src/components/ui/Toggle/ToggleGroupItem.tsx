import { Component, splitProps, createMemo } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useToggleGroup } from './ToggleGroupContext';
import { ToggleGroupItemProperties } from './types';

/**
 * Individual item within a ToggleGroup.
 * Reacts to the parent ToggleGroup state (single or multiple).
 *
 * @param {ToggleGroupItemProperties} properties - Properties for the ToggleGroupItem.
 * @returns {JSX.Element} The rendered toggle group item button.
 *
 * @example
 * <ToggleGroupItem value="left">
 *   <AlignLeft />
 * </ToggleGroupItem>
 */
export const ToggleGroupItem: Component<ToggleGroupItemProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'value',
        'disabled',
        'children'
    ]);

    const context = useToggleGroup();

    /**
     * Determines if this item is currently pressed based on the context value.
     */
    const isPressed = createMemo(() => {
        const currentValue = context.value();
        if (context.type === 'single') {
            return currentValue === localProperties.value;
        }
        return (currentValue as string[]).includes(localProperties.value);
    });

    /**
     * Determines if the item is disabled (either individually or by the group).
     */
    const isDisabled = () => localProperties.disabled || context.disabled;

    /**
     * Handles the click event by delegating to the context's onItemClick handler.
     */
    const handleClick = () => {
        if (isDisabled()) return;
        context.onItemClick(localProperties.value);
    };

    return (
        <button
            type="button"
            class={cn(
                'ui-toggle-group-item',
                `ui-toggle-group-item-${context.size()}`,
                isPressed() && 'ui-toggle-group-item-pressed',
                localProperties.class
            )}
            aria-pressed={isPressed()}
            disabled={isDisabled()}
            onClick={handleClick}
            {...remainingProperties}
        >
            {localProperties.children}
        </button>
    );
};
