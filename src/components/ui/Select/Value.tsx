import { Component, createMemo, Show } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';

/**
 * Properties for Select.Value component.
 */
interface SelectValueProperties {
    /** Placeholder message when no option is selected. */
    placeholder?: string;
    /** Custom CSS class. */
    class?: string;
}

/**
 * Select.Value renders the label of the currently selected option.
 *
 * @param {SelectValueProperties} properties - Properties for the value display.
 * @returns {JSX.Element} A span containing the selected label or placeholder.
 */
export const Value: Component<SelectValueProperties> = properties => {
    const context = useSelect();

    /**
     * Finds and accesses the label of the currently selected value.
     */
    const activeOptionLabel = createMemo(() => {
        const option = context.options().find(opt => opt.value === context.value());
        return option ? option.label : undefined;
    });

    return (
        <span
            class={concatenateClasses(
                'ui-select-value',
                !activeOptionLabel() && 'ui-select-placeholder',
                properties.class
            )}
        >
            <Show when={activeOptionLabel()} fallback={properties.placeholder || 'Select...'}>
                {activeOptionLabel()}
            </Show>
        </span>
    );
};
