import { Component } from 'solid-js';
import { Dynamic } from 'solid-js/web';

interface TreeViewIconProps {
    /** Icon component to render */
    icon: Component<{ size?: number | string; color?: string; fill?: string; stroke?: string }>;
    /** Custom color for the icon */
    color?: string;
}

/**
 * Atomic component for rendering tree node icons.
 */
export const TreeViewIcon: Component<TreeViewIconProps> = props => {
    return (
        <span
            class="ui-tree-icon"
            style={{ color: props.color || 'var(--text-secondary)' }}
            aria-hidden="true"
        >
            <Dynamic component={props.icon} size={14} />
        </span>
    );
};
