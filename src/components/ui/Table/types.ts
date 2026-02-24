import { JSX, Component } from 'solid-js';
import type { LucideProps } from 'lucide-solid';

/**
 * Column definition for the modular Table component.
 * Supports typed data accessors, custom rendering, and layout configuration.
 *
 * @template T - The record type for the table row.
 */
export interface Column<T extends Record<string, unknown>> {
    /** The text or element to display in the header */
    header: string | JSX.Element;
    /** The key in the data object to access the value */
    accessorKey: keyof T | string;
    /** Optional width (e.g. '100px', '20%', or 150) */
    width?: string | number;
    /** Optional custom cell renderer */
    cell?: (item: T) => JSX.Element;
    /** Whether the column is sortable */
    sortable?: boolean;
    /** Whether the column is currently hidden */
    hidden?: boolean;
    /** Optional alignment of cell content */
    align?: 'left' | 'center' | 'right';
    /** Is this a pinned column? (Future-proofing) */
    pinned?: 'left' | 'right';
}

/**
 * Defines the sorting state for the table columns.
 */
export type SortOrder = 'asc' | 'desc' | null;

/**
 * Configuration properties for the Table component.
 * Includes data, column definitions, virtualization settings, and event callbacks.
 *
 * @template T - The record type for the table data.
 */
export interface TableProps<T extends Record<string, unknown>> {
    /** Array of data items to display */
    data: T[];
    /** Column definitions */
    columns: Column<T>[];
    /** Fixed height of each row in pixels (default: 32) */
    rowHeight?: number;
    /** Whether the header should stick to the top when scrolling */
    stickyHeader?: boolean;
    /** Currently active sort key */
    sortKey?: string | null;
    /** Currently active sort order */
    sortOrder?: SortOrder;
    /** List of selected item identifiers */
    selectedIds?: (string | number)[];
    /** Callback when column sort state changes */
    onSort?: (key: string, order: SortOrder) => void;
    /**
     * Callback when a row is clicked.
     * @param item - The clicked data item.
     * @param multi - True if Ctrl/Meta key was pressed (multi-select).
     * @param range - True if Shift key was pressed (range-select).
     */
    onRowClick?: (item: T, multi: boolean, range: boolean) => void;
    /** Callback for double click event on a row */
    onRowDoubleClick?: (item: T) => void;
    /** Callback for scroll events on the table container */
    onScroll?: (e: Event) => void;
    /** Callback when a row element is mounted to the DOM */
    onRowMount?: (el: HTMLElement, item: T) => void;
    /** Key field to use for identifying rows (default: 'id') */
    keyField?: keyof T;
    /** Additional CSS class for the table container */
    class?: string;
    /** Fixed height or CSS height string for the table container (required for virtualization) */
    height?: string | number;
    /** ARIA label for the grid-based interactive table */
    label?: string;
    /** Primary message to display when table has no data */
    emptyMessage?: string;
    /** Detailed description for empty state UI */
    emptyDescription?: string;
    /** Custom Lucide icon component for empty state */
    emptyIcon?: Component<LucideProps>;
    /**
     * Callback when the visible item range changes due to virtualization.
     * Useful for lazy-loading or priority processing of visible items.
     */
    onVisibleItemsChange?: (items: T[]) => void;
}
