import { Component, onMount } from 'solid-js';

/**
 * Properties for the TreeViewInput component.
 */
interface TreeViewInputProps {
    /** The current text value of the node label. */
    value: string;
    /** Callback function invoked when the change is committed (e.g., via Enter or Blur). */
    onCommit: (newValue: string) => void;
    /** Callback function invoked when the edit mode is explicitly cancelled (e.g., via Escape). */
    onCancel: () => void;
    /** Optional callback invoked when the input field receives focus. */
    onFocus?: () => void;
    /** Optional callback invoked when the input field loses focus. */
    onBlur?: () => void;
}

/**
 * Atomic specialized input component for renaming tree nodes in-place.
 *
 * This component automatically handles focus on mount, selects all existing text for quick
 * overwriting, and manages standard keyboard interactions (Enter for commit, Escape for cancel).
 *
 * @param {TreeViewInputProps} props - The properties for configuring the input interaction.
 * @returns {JSX.Element} A stylized HTML input element configured for tree renames.
 */
export const TreeViewInput: Component<TreeViewInputProps> = props => {
    let inputElementReference: HTMLInputElement | undefined;

    onMount(() => {
        if (inputElementReference) {
            inputElementReference.focus();
            inputElementReference.select();
        }
    });

    /**
     * Handles keyboard events specifically for rename logic.
     * Prevents event propagation to the parent tree elements.
     */
    const handleKeyDown = (event: KeyboardEvent) => {
        event.stopPropagation();
        if (event.key === 'Enter') {
            event.preventDefault();
            inputElementReference?.blur();
        } else if (event.key === 'Escape') {
            event.preventDefault();
            props.onCancel();
        }
    };

    /**
     * Logic for committing the name change when the field loses focus.
     */
    const handleBlur = () => {
        props.onBlur?.();
        if (inputElementReference) {
            const trimmedValue = inputElementReference.value.trim();
            props.onCommit(trimmedValue);
        }
    };

    return (
        <input
            ref={inputElementReference}
            type="text"
            class="ui-tree-input"
            value={props.value}
            onClick={event => event.stopPropagation()}
            onKeyDown={handleKeyDown}
            onFocus={() => props.onFocus?.()}
            onBlur={handleBlur}
            aria-label="Rename tree item"
        />
    );
};
