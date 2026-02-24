import { Component, Show } from 'solid-js';
import { ChevronRight, ChevronDown } from 'lucide-solid';

interface TreeViewToggleProps {
    /** Whether the node is currently expanded */
    isExpanded: boolean;
    /** Callback when the toggle button is clicked */
    onClick: (event: MouseEvent) => void;
    /** Accessibility label */
    ariaLabel?: string;
}

/**
 * Atomic component for the expansion/collapse toggle button.
 */
export const TreeViewToggle: Component<TreeViewToggleProps> = props => {
    return (
        <button
            type="button"
            class="ui-tree-toggle"
            onClick={(event: MouseEvent) => props.onClick(event)}
            aria-label={props.ariaLabel || (props.isExpanded ? 'Collapse' : 'Expand')}
            tabindex={-1}
        >
            <Show when={props.isExpanded} fallback={<ChevronRight size={12} />}>
                <ChevronDown size={12} />
            </Show>
        </button>
    );
};
