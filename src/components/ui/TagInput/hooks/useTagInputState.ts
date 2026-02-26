import { createSignal, createMemo } from 'solid-js';
import { TagOption, TagInputProps } from '../types';

/**
 * Hook to manage the internal state of the TagInput component.
 * Handles the list of tags, the current input value, and filtering logic.
 *
 * @param props - The properties passed to the TagInput component.
 * @returns An object containing the current state and action functions.
 */
export const useTagInputState = (props: TagInputProps) => {
    const [inputValue, setInputValue] = createSignal('');
    const [showSuggestions, setShowSuggestions] = createSignal(false);
    const [highlightedSuggestionIndex, setHighlightedSuggestionIndex] = createSignal(-1);

    /**
     * Determines if the user can still add more tags based on the 'max' limit.
     */
    const canAddMoreTags = createMemo(() => {
        if (props.max === undefined) {
            return true;
        }
        return props.value.length < props.max;
    });

    /**
     * Reactive list of tag suggestions filtered by the current input value and selected tags.
     */
    const filteredTagSuggestionsList = createMemo(() => {
        const normalizedInput = inputValue().toLowerCase().trim();
        if (!normalizedInput) {
            return [];
        }

        return (props.suggestions || [])
            .filter(tag => tag.label.toLowerCase().includes(normalizedInput))
            .filter(tag => !props.value.some(selected => selected.id === tag.id))
            .slice(0, 10); // Limit results for performance and UX excellence.
    });

    /**
     * Adds a new tag to the selection list and resets the input.
     *
     * @param tag - The tag option to add.
     */
    const addTag = (tag: TagOption) => {
        if (!canAddMoreTags()) {
            return;
        }
        props.onChange([...props.value, tag]);
        setInputValue('');
        setShowSuggestions(false);
        setHighlightedSuggestionIndex(-1);
    };

    /**
     * Removes a tag from the selection list by its identifier.
     *
     * @param identifier - The ID of the tag to remove.
     */
    const removeTag = (identifier: string | number) => {
        props.onChange(props.value.filter(tag => tag.id !== identifier));
    };

    return {
        inputValue,
        setInputValue,
        showSuggestions,
        setShowSuggestions,
        highlightedSuggestionIndex,
        setHighlightedSuggestionIndex,
        filteredTagSuggestionsList,
        canAddMoreTags,
        addTag,
        removeTag
    };
};
