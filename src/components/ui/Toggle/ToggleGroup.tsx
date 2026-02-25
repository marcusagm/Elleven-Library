import { Component, splitProps, createMemo } from 'solid-js';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { ToggleGroupContext } from './ToggleGroupContext';
import {
    ToggleGroupProperties,
    ToggleGroupSingleProperties,
    ToggleGroupMultipleProperties,
    ToggleGroupContextValue,
    ToggleGroupSize
} from './types';
import './toggle-group.css';

/**
 * ToggleGroup component for selecting one or multiple options.
 * Follows the Mundam design system guidelines for visual excellence and SOLID principles.
 *
 * @param {ToggleGroupProperties} properties - Properties for the ToggleGroup.
 * @returns {JSX.Element} The rendered ToggleGroup container with context.
 *
 * @example
 * // Single selection
 * <ToggleGroup type="single" defaultValue="center">
 *   <ToggleGroupItem value="left"><AlignLeft /></ToggleGroupItem>
 *   <ToggleGroupItem value="center"><AlignCenter /></ToggleGroupItem>
 *   <ToggleGroupItem value="right"><AlignRight /></ToggleGroupItem>
 * </ToggleGroup>
 */
export const ToggleGroup: Component<ToggleGroupProperties> = properties => {
    const [localProperties] = splitProps(properties, [
        'class',
        'type',
        'value',
        'defaultValue',
        'onValueChange',
        'disabled',
        'orientation',
        'size',
        'children'
    ]);

    const isSingleMode = () => localProperties.type === 'single';

    // Single selection state management
    const singleSelectionState = createControllableSignal<string>({
        value: isSingleMode()
            ? () => (localProperties as ToggleGroupSingleProperties).value
            : undefined,
        defaultValue: isSingleMode()
            ? ((localProperties as ToggleGroupSingleProperties).defaultValue ?? '')
            : '',
        onChange: isSingleMode()
            ? (localProperties as ToggleGroupSingleProperties).onValueChange
            : undefined
    });

    // Multiple selection state management
    const multipleSelectionState = createControllableSignal<string[]>({
        value: !isSingleMode()
            ? () => (localProperties as ToggleGroupMultipleProperties).value
            : undefined,
        defaultValue: !isSingleMode()
            ? ((localProperties as ToggleGroupMultipleProperties).defaultValue ?? [])
            : [],
        onChange: !isSingleMode()
            ? (localProperties as ToggleGroupMultipleProperties).onValueChange
            : undefined
    });

    /**
     * Unified accessor for the current value, regardless of mode.
     */
    const activeValue = createMemo(() => {
        if (isSingleMode()) {
            return singleSelectionState.value();
        }
        return multipleSelectionState.value();
    });

    /**
     * Handles single mode click.
     */
    const handleSingleClick = (itemValue: string) => {
        const current = singleSelectionState.value();
        singleSelectionState.setValue(current === itemValue ? '' : itemValue);
    };

    /**
     * Handles multiple mode click.
     */
    const handleMultipleClick = (itemValue: string) => {
        const current = multipleSelectionState.value();
        if (current.includes(itemValue)) {
            multipleSelectionState.setValue(current.filter((value: string) => value !== itemValue));
        } else {
            multipleSelectionState.setValue([...current, itemValue]);
        }
    };

    /**
     * Handles item clicks by updating the appropriate selection state.
     * @param {string} itemValue - The value of the clicked item.
     */
    const handleItemClick = (itemValue: string) => {
        if (localProperties.disabled) return;

        if (isSingleMode()) {
            handleSingleClick(itemValue);
        } else {
            handleMultipleClick(itemValue);
        }
    };

    /**
     * Data object to be shared via context.
     */
    const contextValue: ToggleGroupContextValue = {
        get type() {
            return localProperties.type;
        },
        value: activeValue,
        onItemClick: handleItemClick,
        get disabled() {
            return localProperties.disabled ?? false;
        },
        size: () => localProperties.size || ('md' as ToggleGroupSize)
    };

    return (
        <ToggleGroupContext.Provider value={contextValue}>
            <div
                class={cn(
                    'ui-toggle-group',
                    `ui-toggle-group-${localProperties.orientation || 'horizontal'}`,
                    localProperties.class
                )}
                role="group"
                aria-disabled={localProperties.disabled}
            >
                {localProperties.children}
            </div>
        </ToggleGroupContext.Provider>
    );
};
