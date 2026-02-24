import { Component } from 'solid-js';

/**
 * Properties for the TreeViewLabel component.
 */
interface TreeViewLabelProps {
    /** The human-readable text content to display for the node. */
    text: string;
}

/**
 * Atomic presentational component for displaying the main text label of a tree node.
 *
 * @param {TreeViewLabelProps} props - The properties for the label text.
 * @returns {JSX.Element} A stylized span container for the label text.
 */
export const TreeViewLabel: Component<TreeViewLabelProps> = props => {
    return <span class="ui-tree-label">{props.text}</span>;
};
