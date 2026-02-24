import { Component } from 'solid-js';
import { Dynamic } from 'solid-js/web';

/**
 * Properties for the TreeViewIcon component.
 */
interface TreeViewIconProps {
    /**
     * The icon component to render.
     * Expects a component that follows the common icon prop pattern (size, color, etc.).
     */
    icon: Component<{ size?: number | string; color?: string; fill?: string; stroke?: string }>;
    /** Optional custom color for the icon. Defaults to the secondary text color if not provided. */
    color?: string;
}

/**
 * Atomic component for rendering tree node icons with consistent sizing and alignment.
 *
 * @param {TreeViewIconProps} props - The properties for the icon component and its styling.
 * @returns {JSX.Element} A containerized icon element.
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
