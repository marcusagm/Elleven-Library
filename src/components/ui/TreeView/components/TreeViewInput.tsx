import { Component, onMount } from 'solid-js';

interface TreeViewInputProps {
    /** Current label value */
    value: string;
    /** Callback when change is committed (usually on blur or Enter) */
    onCommit: (newValue: string) => void;
    /** Callback when edit is cancelled */
    onCancel: () => void;
    /** Callback when input receives focus */
    onFocus?: () => void;
    /** Callback when input loses focus */
    onBlur?: () => void;
}

/**
 * Atomic specialized input for renaming tree nodes.
 */
export const TreeViewInput: Component<TreeViewInputProps> = props => {
    let textInputReference: HTMLInputElement | undefined;

    onMount(() => {
        if (textInputReference) {
            textInputReference.focus();
            textInputReference.select();
        }
    });

    const handleKeyDown = (event: KeyboardEvent) => {
        event.stopPropagation();
        if (event.key === 'Enter') {
            event.preventDefault();
            textInputReference?.blur();
        } else if (event.key === 'Escape') {
            event.preventDefault();
            props.onCancel();
        }
    };

    const handleBlur = () => {
        props.onBlur?.();
        if (textInputReference) {
            const trimmedValue = textInputReference.value.trim();
            props.onCommit(trimmedValue);
        }
    };

    return (
        <input
            ref={textInputReference}
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
