import { Component, createMemo, Show } from 'solid-js';
import { Check as CheckIcon } from 'lucide-solid';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';
import { SelectItemProperties } from './types';

/**
 * Select.Item represents a single selectable entry within the Select.Content.
 *
 * @param {SelectItemProperties} properties - Properties for the item entry.
 * @returns {JSX.Element} A stylized option item.
 */
export const Item: Component<SelectItemProperties> = properties => {
    const context = useSelect();

    /**
     * Reactively checks if this item is currently selected.
     */
    const isSelectedResult = createMemo(() => context.value() === properties.option.value);

    /**
     * Determines the index of this item in the options list for highlighting.
     */
    const itemIndex = createMemo(() => context.options().indexOf(properties.option));

    /**
     * Updates the global selection within the group.
     */
    const handleSelectionTrigger = () => {
        if (properties.option.disabled) {
            return;
        }
        context.setValue(properties.option.value);
        context.setIsOpen(false);
    };

    /**
     * Updates the highlight status on hover for keyboard navigation.
     */
    const handleNavigationHint = () => {
        if (properties.option.disabled) {
            return;
        }
        context.setHighlightedIndex(itemIndex());
    };

    return (
        <div
            class={concatenateClasses(
                'ui-select-option',
                isSelectedResult() && 'ui-select-option-selected',
                properties.option.disabled && 'ui-select-option-disabled',
                context.highlightedIndex() === itemIndex() && 'ui-select-option-highlighted',
                properties.class
            )}
            role="option"
            aria-selected={isSelectedResult()}
            aria-disabled={properties.option.disabled}
            onClick={handleSelectionTrigger}
            onMouseEnter={handleNavigationHint}
        >
            <span class="ui-select-option-label">{properties.option.label}</span>
            <Show when={isSelectedResult()}>
                <CheckIcon size={14} class="ui-select-check" />
            </Show>
        </div>
    );
};
