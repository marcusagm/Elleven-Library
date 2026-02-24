import { Component } from 'solid-js';

interface TreeViewLabelProps {
    /** Text content to display */
    text: string;
}

/**
 * Atomic component for rendering the tree node label.
 */
export const TreeViewLabel: Component<TreeViewLabelProps> = props => {
    return <span class="ui-tree-label">{props.text}</span>;
};
