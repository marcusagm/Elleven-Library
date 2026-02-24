import { JSX, Component } from 'solid-js';
import { DragItem } from '../../../core/dnd';

/**
 * Valid relative positions for a drop operation within the tree.
 */
export type TreeDropPosition = 'before' | 'inside' | 'after';

/**
 * Generic tree node structure.
 * @template T - Type of the additional data stored in the node.
 */
export interface TreeNode<T = unknown> {
    /**
     * Unique identifier for the node.
     * Must be unique within the entire tree.
     */
    id: string | number;

    /**
     * Human-readable text displayed for the node.
     */
    label: string;

    /**
     * Optional icon component to display for the node.
     */
    icon?: Component;

    /**
     * Optional specific color for the icon.
     * This is used to override the default icon color.
     */
    iconColor?: string;
    /** Nested child nodes */
    children?: TreeNode<T>[];
    /** Optional secondary element (badge, count, etc.) to display next to the label */
    badge?: JSX.Element;
    /** Additional business logic data */
    data?: T;
}

/**
 * Shared properties for the TreeView system.
 * @template T - Type of the business data in nodes.
 */
export interface TreeViewProps<T = unknown> {
    /** Hierarchical list of nodes to display */
    items: TreeNode<T>[];
    /** CSS class for the root container */
    class?: string;
    /** Indentation distance for each nesting level (default: 16) */
    indentSize?: number;
    /** IDs of nodes that are currently expanded */
    expandedIds?: Set<string | number>;
    /** IDs of nodes that are currently selected */
    selectedIds?: (string | number)[];
    /** ID of the node currently in rename mode */
    editingId?: string | number | null;
    /** Default icon to use if a node doesn't specify one */
    defaultIcon?: Component;
    /** Whether nodes can be dragged */
    draggable?: boolean;
    /** The specific drag-and-drop type identifier for this tree instance (e.g., 'TAG', 'FOLDER') */
    dragType?: string;
    /** List of drag types this tree accepts (e.g., ['TAG', 'IMAGE']) */
    acceptedDragTypes?: string[];

    // Callbacks
    /** Triggered when a node is clicked */
    onSelect?: (node: TreeNode<T>) => void;
    /** Triggered when a node is toggled (expanded/collapsed) */
    onToggle?: (id: string | number) => void;
    /** Triggered when right-clicking a node */
    onContextMenu?: (event: MouseEvent, node: TreeNode<T>) => void;
    /** Triggered when a rename operation is committed */
    onRename?: (node: TreeNode<T>, newLabel: string) => void;
    /** Triggered when rename mode is exited without saving */
    onEditCancel?: () => void;
    /** Triggered when a node is moved via drag-and-drop */
    onMove?: (node: TreeNode<T>, target: TreeNode<T> | 'root', position: TreeDropPosition) => void;
    /** Optional custom validation for drop operations */
    isValidDrop?: (dragged: DragItem, target: TreeNode<T>) => boolean;
}

/**
 * Internal properties for the TreeViewItem component,
 * including depth tracking and layout context.
 */
export interface TreeViewItemProps<T = unknown> extends TreeViewProps<T> {
    /** The specific node instance to render */
    node: TreeNode<T>;
    /** Nesting depth (0-indexed) */
    depth: number;
    /** Reference ID for the parent tree container */
    treeId: string;
    /** Whether this is the last sibling in its group (for guide lines) */
    isLast: boolean;
}
