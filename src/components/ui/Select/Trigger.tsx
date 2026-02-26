import { Component, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';
import { SelectTriggerProperties } from './types';

/**
 * Select.Trigger is the interactive button that opens the Select dropdown.
 *
 * @param {SelectTriggerProperties} properties - Properties for the trigger button.
 * @returns {JSX.Element} The stylized button that toggles open state.
 */
export const Trigger: Component<SelectTriggerProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'children',
        'error',
        'size'
    ]);

    const context = useSelect();

    /**
     * Toggles the visibility of the dropdown when the trigger is clicked.
     */
    const handleTriggerToggle = () => {
        if (context.disabled()) {
            return;
        }
        context.setIsOpen(!context.isOpen());

        // Reset search and highlight state when opening the dropdown.
        if (context.isOpen()) {
            context.setSearchQuery('');
            context.setHighlightedIndex(-1);
        }
    };

    /**
     * Accessibility logic for keyboard interactions (Enter, Space, ArrowDown).
     */
    const handleKeyboardInput = (event: KeyboardEvent) => {
        if (!context.isOpen()) {
            if (event.key === 'Enter' || event.key === ' ' || event.key === 'ArrowDown') {
                event.preventDefault();
                handleTriggerToggle();
            }
        }
    };

    const activeSize = () => localProperties.size || 'md';

    return (
        <button
            ref={element => context.setTriggerElement(element)}
            type="button"
            class={concatenateClasses(
                'ui-select-trigger',
                `ui-select-trigger-${activeSize()}`,
                context.isOpen() && 'ui-select-trigger-open',
                context.disabled() && 'ui-select-trigger-disabled',
                localProperties.error && 'ui-select-trigger-error',
                localProperties.class
            )}
            role="combobox"
            aria-expanded={context.isOpen()}
            aria-haspopup="listbox"
            aria-disabled={context.disabled()}
            disabled={context.disabled()}
            onClick={handleTriggerToggle}
            onKeyDown={handleKeyboardInput}
            {...remainingProperties}
        >
            {localProperties.children}
        </button>
    );
};
