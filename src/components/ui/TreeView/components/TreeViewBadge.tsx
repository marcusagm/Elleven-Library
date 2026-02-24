import { Component, JSX } from 'solid-js';

interface TreeViewBadgeProps {
    /** Badge content */
    children: JSX.Element;
}

/**
 * Atomic wrapper for tree node badges or supplementary elements.
 */
export const TreeViewBadge: Component<TreeViewBadgeProps> = props => {
    return <span class="ui-tree-badge">{props.children}</span>;
};
