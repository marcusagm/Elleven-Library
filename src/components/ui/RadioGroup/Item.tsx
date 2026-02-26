import { Component, splitProps, createMemo, Show } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { createId as generateUniqueId } from '../../../lib/primitives/createId';
import { useRadioGroup } from './context';
import { RadioGroupItemProperties } from './types';

/**
 * RadioGroupItem represents a single selectable option in a RadioGroup.
 * Must be used within a RadioGroup component.
 *
 * @param {RadioGroupItemProperties} properties - Properties for the radio item.
 * @returns {JSX.Element} A stylized radio input and label.
 *
 * @example
 * <RadioGroupItem value="option-A" label="Option A" />
 */
export const RadioGroupItem: Component<RadioGroupItemProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'value',
        'label',
        'description',
        'id',
        'disabled',
        'size'
    ]);

    const context = useRadioGroup();

    /**
     * Unique accessible identifier for the radio item.
     */
    const accessibleId = createMemo(() => localProperties.id || generateUniqueId('radio'));

    /**
     * Unique ID for the description to link it to the radio.
     */
    const descriptionIdentifier = createMemo(() =>
        localProperties.description ? `${accessibleId()}-description` : undefined
    );

    /**
     * Reactively derived status of the current item within its group.
     */
    const isSelected = createMemo(() => context.value() === localProperties.value);

    /**
     * Determines if the item is disabled based on its props or group context.
     */
    const isGloballyDisabled = () => localProperties.disabled || context.disabled;

    /**
     * Notifies the group about a selection choice.
     */
    const handleSelectionTrigger = () => {
        if (isGloballyDisabled()) {
            return;
        }
        context.onChange(localProperties.value);
    };

    /**
     * Accessibility support for keyboard interaction (Space/Enter).
     */
    const handleKeyboardInput = (event: KeyboardEvent) => {
        if (isGloballyDisabled()) {
            return;
        }
        if (event.key === ' ' || event.key === 'Enter') {
            event.preventDefault();
            context.onChange(localProperties.value);
        }
    };

    const activeSize = () => localProperties.size || 'md';

    return (
        <label
            class={concatenateClasses(
                'ui-radio-wrapper',
                isGloballyDisabled() && 'ui-radio-disabled',
                localProperties.class
            )}
            for={accessibleId()}
        >
            <button
                type="button"
                role="radio"
                id={accessibleId()}
                class={concatenateClasses(
                    'ui-radio',
                    `ui-radio-${activeSize()}`,
                    isSelected() && 'ui-radio-checked'
                )}
                aria-checked={isSelected()}
                aria-disabled={isGloballyDisabled()}
                aria-describedby={descriptionIdentifier()}
                disabled={isGloballyDisabled()}
                onClick={handleSelectionTrigger}
                onKeyDown={handleKeyboardInput}
            >
                <span class="ui-radio-indicator" />
            </button>

            <Show when={localProperties.label || localProperties.description}>
                <div class="ui-radio-content">
                    <Show when={localProperties.label}>
                        <span
                            class={concatenateClasses(
                                'ui-radio-label',
                                `ui-radio-label-${activeSize()}`
                            )}
                        >
                            {localProperties.label}
                        </span>
                    </Show>
                    <Show when={localProperties.description}>
                        <span
                            id={descriptionIdentifier()}
                            class={concatenateClasses(
                                'ui-radio-description',
                                `ui-radio-description-${activeSize()}`
                            )}
                        >
                            {localProperties.description}
                        </span>
                    </Show>
                </div>
            </Show>

            <input
                type="radio"
                name={context.name}
                value={localProperties.value}
                checked={isSelected()}
                disabled={isGloballyDisabled()}
                class="ui-radio-input"
                tabindex={-1}
                aria-hidden="true"
                {...remainingProperties}
            />
        </label>
    );
};
