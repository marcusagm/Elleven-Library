import { Component, For } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn } from '../../../lib/utils';
import { TagOption } from './types';

/**
 * Properties for the TagSuggestions component.
 */
interface TagSuggestionsProps {
    /** List of filtered suggestions to display */
    suggestions: TagOption[];
    /** The index of the currently highlighted suggestion */
    highlightedIndex: number;
    /** Unique identifier for the listbox for accessibility */
    listboxId: string;
    /** Current position and dimension for the fixed overlay */
    position: { top: number; left: number; width: number };
    /** Callback when a suggestion is clicked */
    onSelect: (tag: TagOption) => void;
    /** Callback when mouse enters a suggestion item */
    onHighlight: (index: number) => void;
    /** Optional ref for the dropdown element */
    ref?: HTMLUListElement | ((el: HTMLUListElement) => void);
}

/**
 * A portal-based dropdown menu that displays autocomplete suggestions for the TagInput.
 *
 * @param props - Properties for the TagSuggestions component.
 * @returns The rendered TagSuggestions component.
 */
export const TagSuggestions: Component<TagSuggestionsProps> = props => {
    return (
        <Portal>
            <ul
                ref={props.ref}
                id={props.listboxId}
                role="listbox"
                class="ui-tag-suggestions"
                onMouseDown={event => event.preventDefault()}
                style={{
                    position: 'fixed',
                    top: `${props.position.top}px`,
                    left: `${props.position.left}px`,
                    width: `${props.position.width}px`,
                    'z-index': 9999
                }}
            >
                <For each={props.suggestions}>
                    {(tagSuggestion, index) => (
                        <li
                            id={`${props.listboxId}-option-${index()}`}
                            role="option"
                            class={cn(
                                'ui-tag-suggestion-item',
                                props.highlightedIndex === index() &&
                                    'ui-tag-suggestion-highlighted'
                            )}
                            aria-selected={props.highlightedIndex === index()}
                            onClick={() => props.onSelect(tagSuggestion)}
                            onMouseEnter={() => props.onHighlight(index())}
                        >
                            {tagSuggestion.label}
                        </li>
                    )}
                </For>
            </ul>
        </Portal>
    );
};
