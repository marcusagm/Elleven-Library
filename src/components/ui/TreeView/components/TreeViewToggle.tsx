import { Component, Show } from 'solid-js';
import { ChevronRight, ChevronDown } from 'lucide-solid';

/**
 * Properties for the TreeViewToggle component.
 */
interface TreeViewToggleProps {
    /** Accessor or value indicating whether the target node is currently expanded. */
    isExpanded: boolean;
    /** Callback function invoked when the toggle button is clicked (used to expand/collapse). */
    onClick: (event: MouseEvent) => void;
    /** Optional accessibility label for the button. Defaults to 'Expand' or 'Collapse' based on state. */
    ariaLabel?: string;
}

/**
 * Atomic button component for toggling the expansion and collapse of hierarchical tree nodes.
 *
 * @param {TreeViewToggleProps} props - The properties for configuring the toggle button.
 * @returns {JSX.Element} A stylized button element containing a reactive chevron icon.
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
