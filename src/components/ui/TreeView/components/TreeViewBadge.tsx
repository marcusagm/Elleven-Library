import { Component, JSX } from 'solid-js';

/**
 * Properties for the TreeViewBadge component.
 */
interface TreeViewBadgeProps {
    /** The dynamic content to be displayed within the badge (e.g., a count or status icon). */
    children: JSX.Element;
}

/**
 * Atomic presentational component for displaying badges or supplementary information next to tree nodes.
 *
 * @param {TreeViewBadgeProps} props - The properties for the badge content.
 * @returns {JSX.Element} A stylized span container for the badge content.
 */
export const TreeViewBadge: Component<TreeViewBadgeProps> = props => {
    return <span class="ui-tree-badge">{props.children}</span>;
};
