import { Component, splitProps, createMemo } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { createId as generateUniqueId } from '../../../lib/primitives/createId';
import { RadioGroupContext } from './context';
import { RadioGroupProperties } from './types';
import './radio-group.css';

/**
 * RadioGroup component for selecting a single option from a list.
 *
 * @param {RadioGroupProperties} properties - Properties for the group container.
 * @returns {JSX.Element} The group wrapper with context provider.
 *
 * @example
 * <RadioGroup defaultValue="option1" onValueChange={console.log}>
 *   <RadioGroupItem value="option1" label="Option 1" />
 *   <RadioGroupItem value="option2" label="Option 2" />
 * </RadioGroup>
 */
export const RadioGroup: Component<RadioGroupProperties> = properties => {
    const [localProperties] = splitProps(properties, [
        'class',
        'value',
        'defaultValue',
        'onValueChange',
        'name',
        'disabled',
        'orientation',
        'children'
    ]);

    /**
     * Generates a unique group name for the radios if none provided.
     */
    const groupName = createMemo(() => localProperties.name || generateUniqueId('radio-group'));

    /**
     * Internal signal to manage the selected radio value.
     */
    const { value: selectedValue, setValue: setSelectedValue } = createControllableSignal({
        value: () => localProperties.value,
        defaultValue: localProperties.defaultValue ?? '',
        onChange: (value: string) => localProperties.onValueChange?.(value)
    });

    /**
     * Shared context provider for child RadioGroupItem components.
     */
    const contextValue = {
        get name() {
            return groupName();
        },
        value: selectedValue,
        onChange: setSelectedValue,
        get disabled() {
            return localProperties.disabled ?? false;
        }
    };

    return (
        <RadioGroupContext.Provider value={contextValue}>
            <div
                class={concatenateClasses(
                    'ui-radio-group',
                    `ui-radio-group-${localProperties.orientation || 'vertical'}`,
                    localProperties.class
                )}
                role="radiogroup"
                aria-disabled={localProperties.disabled}
            >
                {localProperties.children}
            </div>
        </RadioGroupContext.Provider>
    );
};
