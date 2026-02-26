import { Component, splitProps, createEffect, JSX } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';

/**
 * Properties for Select.Search component.
 */
interface SelectSearchProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'value' | 'onInput'
> {
    /** Additional CSS class. */
    class?: string;
}

/**
 * Select.Search provides a filtering input within the Select.Content component.
 *
 * @param {SelectSearchProperties} properties - Input properties.
 * @returns {JSX.Element} A stylized filtering input.
 */
export const Search: Component<SelectSearchProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, ['class']);

    const context = useSelect();
    let searchInputRef: HTMLInputElement | undefined;

    /**
     * Ensures search input is focused as soon as the dropdown is opened.
     */
    createEffect(() => {
        if (context.isOpen() && searchInputRef) {
            searchInputRef.focus();
        }
    });

    /**
     * Logic to handle user typing into the search field.
     */
    const handleInputUpdate = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const queryValue = event.currentTarget.value;
        context.setSearchQuery(queryValue);
        context.setHighlightedIndex(0);
    };

    /**
     * Keyboard logic (Escape, Enter) forwarded from search.
     */
    const handleKeyboardInput = (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
            event.preventDefault();
            context.setIsOpen(false);
        }
    };

    return (
        <div class="ui-select-search">
            <input
                ref={searchInputRef}
                type="text"
                class={concatenateClasses('ui-select-search-input', localProperties.class)}
                placeholder="Search..."
                value={context.searchQuery()}
                onInput={handleInputUpdate}
                onKeyDown={handleKeyboardInput}
                {...remainingProperties}
            />
        </div>
    );
};
