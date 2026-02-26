/* eslint-disable complexity */
import {
    Component,
    For,
    Show,
    createSignal,
    createEffect,
    onCleanup,
    splitProps,
    createMemo
} from 'solid-js';
import { cn } from '../../../lib/utils';
import { createId } from '../../../lib/primitives/createId';
import { createClickOutside } from '../../../lib/primitives';
import { useInputEvents } from '../Input/useInputEvents';
import { TagOption, TagInputProps } from './types';
import { TagChip } from './TagChip';
import { TagSuggestions } from './TagSuggestions';
import './tag-input.css';

/**
 * TagInput component for managing a list of tags with autocomplete support.
 * Integrates with the core input system for keyboard shortcut safety and accessibility.
 *
 * @param props - Properties for the TagInput component.
 * @returns The rendered TagInput component.
 */
export const TagInput: Component<TagInputProps> = props => {
    // Separate specialized tag logic properties from standard attributes.
    // We strictly avoid abbreviations.
    const [localTagInputProperties] = splitProps(props, [
        'value',
        'onChange',
        'placeholder',
        'suggestions',
        'onCreate',
        'disabled',
        'max',
        'class'
    ]);

    const [inputValue, setInputValue] = createSignal('');
    const [showSuggestions, setShowSuggestions] = createSignal(false);
    const [highlightedSuggestionIndex, setHighlightedSuggestionIndex] = createSignal(-1);
    const [suggestionDropdownPosition, setSuggestionDropdownPosition] = createSignal({
        top: 0,
        left: 0,
        width: 0
    });

    let inputContainerReference: HTMLDivElement | undefined;
    let inputFieldReference: HTMLInputElement | undefined;
    let dropdownListReference: HTMLUListElement | undefined;

    const uniqueIdentifier = createId('tag-input');
    const suggestionListboxIdentifier = `${uniqueIdentifier}-listbox`;

    /**
     * Determines if the user can still add more tags.
     */
    const canAddMoreTags = () => {
        if (localTagInputProperties.max === undefined) {
            return true;
        }
        return localTagInputProperties.value.length < localTagInputProperties.max;
    };

    /**
     * Filters available suggestions based on the current input value and selected tags.
     */
    const filteredTagSuggestionsList = createMemo(() => {
        const normalizedInput = inputValue().toLowerCase().trim();
        if (!normalizedInput) {
            return [];
        }

        return (localTagInputProperties.suggestions || [])
            .filter(tag => tag.label.toLowerCase().includes(normalizedInput))
            .filter(tag => !localTagInputProperties.value.some(selected => selected.id === tag.id))
            .slice(0, 10); // Limit results for performance and UX.
    });

    /**
     * Calculates and updates the position of the suggestions dropdown portal.
     */
    const updateDropdownPosition = () => {
        if (!inputContainerReference || !showSuggestions()) {
            return;
        }

        const containerBoundingClientRect = inputContainerReference.getBoundingClientRect();
        const viewportHeight = window.innerHeight;
        const verticalOffset = 4;

        let topPosition = containerBoundingClientRect.bottom + verticalOffset;
        const leftPosition = containerBoundingClientRect.left;

        // Note: contentRect calculation is tricky with Portal before render.
        // We use a fallback if dropdown is not yet ready or measured.
        const dropdownHeight = 250; // Approximated max height for boundary checks.

        if (topPosition + dropdownHeight > viewportHeight - 10) {
            const spaceAbove = containerBoundingClientRect.top - verticalOffset;
            if (spaceAbove > dropdownHeight) {
                topPosition = containerBoundingClientRect.top - dropdownHeight - verticalOffset;
            } else {
                topPosition = Math.max(10, viewportHeight - dropdownHeight - 10);
            }
        }

        setSuggestionDropdownPosition({
            top: topPosition,
            left: leftPosition,
            width: containerBoundingClientRect.width
        });
    };

    // Use our custom core hook for shortcut safety and focus management.
    const {
        handleFocus,
        handleBlur,
        handleKeyDown: handleCoreKeyDown
    } = useInputEvents({
        onFocus: () => setShowSuggestions(true)
    });

    // Handle clicks outside the component to close suggestions.
    createClickOutside(
        () => [inputContainerReference, dropdownListReference].filter(Boolean) as HTMLElement[],
        () => {
            if (showSuggestions()) {
                setShowSuggestions(false);
            }
        }
    );

    // Dynamic positioning on scroll or resize.
    createEffect(() => {
        const isVisible = showSuggestions() && filteredTagSuggestionsList().length > 0;
        if (isVisible) {
            requestAnimationFrame(updateDropdownPosition);

            const updateHandler = () => requestAnimationFrame(updateDropdownPosition);
            window.addEventListener('scroll', updateHandler, true);
            window.addEventListener('resize', updateHandler);

            onCleanup(() => {
                window.removeEventListener('scroll', updateHandler, true);
                window.removeEventListener('resize', updateHandler);
            });
        }
    });

    /**
     * Handles local keyboard navigation within the tag input.
     *
     * @param event - The keyboard event object.
     */
    const handleLocalKeyDown = (event: KeyboardEvent) => {
        const currentSuggestions = filteredTagSuggestionsList();

        switch (event.key) {
            case 'ArrowDown':
                if (currentSuggestions.length > 0) {
                    setHighlightedSuggestionIndex(previous =>
                        previous < currentSuggestions.length - 1 ? previous + 1 : 0
                    );
                }
                break;

            case 'ArrowUp':
                if (currentSuggestions.length > 0) {
                    setHighlightedSuggestionIndex(previous =>
                        previous > 0 ? previous - 1 : currentSuggestions.length - 1
                    );
                }
                break;

            case 'Enter': {
                const trimmedInput = inputValue().trim();
                if (!trimmedInput || !canAddMoreTags()) {
                    return;
                }

                const highlightedTag = currentSuggestions[highlightedSuggestionIndex()];
                if (highlightedSuggestionIndex() >= 0 && highlightedTag) {
                    addTag(highlightedTag);
                } else {
                    const exactMatch = currentSuggestions.find(
                        tag => tag.label.toLowerCase() === trimmedInput.toLowerCase()
                    );

                    if (exactMatch) {
                        addTag(exactMatch);
                    } else if (localTagInputProperties.onCreate) {
                        localTagInputProperties.onCreate(trimmedInput);
                        setInputValue('');
                        setShowSuggestions(false);
                    }
                }
                setHighlightedSuggestionIndex(-1);
                break;
            }

            case 'Backspace':
                if (!inputValue() && localTagInputProperties.value.length > 0) {
                    const updatedTagList = [...localTagInputProperties.value];
                    updatedTagList.pop();
                    localTagInputProperties.onChange(updatedTagList);
                }
                break;

            case 'Escape':
                setShowSuggestions(false);
                setHighlightedSuggestionIndex(-1);
                break;
        }

        // Forward to core handler for shortcut safety
        handleCoreKeyDown(event);
    };

    /**
     * Adds a new tag to the selection list.
     *
     * @param tag - The tag option to add.
     */
    const addTag = (tag: TagOption) => {
        if (!canAddMoreTags()) {
            return;
        }
        localTagInputProperties.onChange([...localTagInputProperties.value, tag]);
        setInputValue('');
        setShowSuggestions(false);
        setHighlightedSuggestionIndex(-1);
        inputFieldReference?.focus();
    };

    /**
     * Removes a tag from the selection list by its identifier.
     *
     * @param identifier - The ID of the tag to remove.
     */
    const removeTag = (identifier: string | number) => {
        localTagInputProperties.onChange(
            localTagInputProperties.value.filter(tag => tag.id !== identifier)
        );
        inputFieldReference?.focus();
    };

    return (
        <div class={cn('ui-tag-input-wrapper', localTagInputProperties.class)}>
            <div
                ref={inputContainerReference}
                class={cn(
                    'ui-tag-input-container',
                    localTagInputProperties.disabled && 'ui-tag-input-disabled'
                )}
                onClick={() => inputFieldReference?.focus()}
            >
                <For each={localTagInputProperties.value}>
                    {tag => (
                        <TagChip
                            tag={tag}
                            onRemove={removeTag}
                            disabled={localTagInputProperties.disabled}
                        />
                    )}
                </For>

                <input
                    ref={inputFieldReference}
                    type="text"
                    class="ui-tag-input"
                    value={inputValue()}
                    onInput={event => {
                        setInputValue(event.currentTarget.value);
                        setShowSuggestions(true);
                        setHighlightedSuggestionIndex(-1);
                    }}
                    onKeyDown={handleLocalKeyDown}
                    onFocus={handleFocus}
                    onBlur={handleBlur}
                    placeholder={
                        localTagInputProperties.value.length === 0
                            ? localTagInputProperties.placeholder
                            : ''
                    }
                    disabled={localTagInputProperties.disabled || !canAddMoreTags()}
                    aria-autocomplete="list"
                    aria-controls={showSuggestions() ? suggestionListboxIdentifier : undefined}
                    aria-activedescendant={
                        highlightedSuggestionIndex() >= 0
                            ? `${suggestionListboxIdentifier}-option-${highlightedSuggestionIndex()}`
                            : undefined
                    }
                    role="combobox"
                    aria-expanded={showSuggestions()}
                    aria-haspopup="listbox"
                />
            </div>

            <Show when={showSuggestions() && filteredTagSuggestionsList().length > 0}>
                <TagSuggestions
                    suggestions={filteredTagSuggestionsList()}
                    highlightedIndex={highlightedSuggestionIndex()}
                    listboxId={suggestionListboxIdentifier}
                    position={suggestionDropdownPosition()}
                    onSelect={addTag}
                    onHighlight={setHighlightedSuggestionIndex}
                    ref={dropdownListReference}
                />
            </Show>
        </div>
    );
};
