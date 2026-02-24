import { For, Show, createMemo } from 'solid-js';
import { ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import type { Column, SortOrder } from './types';

/**
 * Properties for the TableHeader component.
 */
interface TableHeaderProps<T> {
    /** Set of column definitions used to render the headers */
    columns: Column<T>[];
    /** The key of the column currently being sorted */
    sortKey: string | null | undefined;
    /** The current active sorting order (ascending or descending) */
    sortOrder: SortOrder | undefined;
    /** Whether the header should remain fixed at the top during scrolling */
    sticky: boolean;
    /** Callback triggered when a sortable header is clicked */
    onSort: (columnKey: string) => void;
    /** Callback triggered when a column is resized by the user */
    onColumnResize?: (columnKey: string, newWidth: number) => void;
    /** Callback triggered when the header row is right-clicked */
    onHeaderContextMenu?: (event: MouseEvent) => void;
}

/**
 * Properties for the SortIcon helper component.
 */
interface SortIconProps {
    /** Whether sorting is active for the current column */
    active: boolean;
    /** The active direction of the sort */
    order: SortOrder | undefined;
}

/**
 * Internal helper component to render the sorting state icon.
 * Displays up, down, or default dual-direction icons based on sort state.
 *
 * @param {SortIconProps} props - Sort state properties.
 * @returns {JSX.Element} The rendered sort indicator icon.
 */
function SortIcon(props: SortIconProps) {
    return (
        <span class="ui-table-grid-sort-icon">
            <Show when={props.active} fallback={<ChevronsUpDown size={12} />}>
                <Show when={props.order === 'asc'} fallback={<ChevronDown size={12} />}>
                    <ChevronUp size={12} />
                </Show>
            </Show>
        </span>
    );
}

/**
 * Renders the interactive header row of the table.
 *
 * Responsible for displaying column titles, handling sort interaction,
 * and managing the logic for user-driven column width resizing.
 *
 * @template T - The record type for the table data.
 * @param {TableHeaderProps<T>} props - Header properties and interaction callbacks.
 * @returns {JSX.Element} The rendered interactive header row.
 */
export function TableHeader<T>(props: TableHeaderProps<T>) {
    /**
     * Initializes the column resizing process using pointer capture.
     *
     * @param {Column<T>} targetColumn - The column being resized.
     * @param {PointerEvent} initialEvent - The pointer down event that started resizing.
     */
    const handleResizeStart = (targetColumn: Column<T>, initialEvent: PointerEvent) => {
        initialEvent.preventDefault();
        initialEvent.stopPropagation();

        const initialPointerX = initialEvent.clientX;
        const initialColumnWidth =
            typeof targetColumn.width === 'number'
                ? targetColumn.width
                : parseInt(String(targetColumn.width || '150'));

        /** Handles pointer movement to update column width in real-time */
        const handlePointerMove = (moveEvent: PointerEvent) => {
            const currentPointerX = moveEvent.clientX;
            const horizontalDelta = currentPointerX - initialPointerX;
            /** Ensure the new width respects the configured minimum boundary */
            const computedNewWidth = Math.max(
                targetColumn.minWidth || 50,
                initialColumnWidth + horizontalDelta
            );

            props.onColumnResize?.(targetColumn.accessorKey as string, computedNewWidth);
        };

        /** Finalizes the resize operation and cleans up event listeners */
        const handlePointerUp = () => {
            window.removeEventListener('pointermove', handlePointerMove);
            window.removeEventListener('pointerup', handlePointerUp);
            document.body.style.cursor = '';
        };

        window.addEventListener('pointermove', handlePointerMove);
        window.addEventListener('pointerup', handlePointerUp);
        document.body.style.cursor = 'col-resize';
    };

    return (
        <div
            class={cn('ui-table-grid-header', props.sticky && 'ui-table-grid-header-sticky')}
            role="row"
            aria-rowindex={1}
            onContextMenu={event => props.onHeaderContextMenu?.(event)}
        >
            <For each={props.columns}>
                {column => {
                    /** Whether this specific column is currently sorted */
                    const isColumnActive = createMemo(() => props.sortKey === column.accessorKey);

                    return (
                        <div
                            class={cn(
                                'ui-table-grid-th',
                                column.sortable && 'ui-table-grid-th-sortable',
                                isColumnActive() && 'ui-table-grid-th-active'
                            )}
                            style={{
                                width:
                                    typeof column.width === 'number'
                                        ? `${column.width}px`
                                        : column.width || '150px',
                                flex: column.width ? '0 0 auto' : '1 1 0',
                                'justify-content':
                                    column.align === 'center'
                                        ? 'center'
                                        : column.align === 'right'
                                          ? 'flex-end'
                                          : 'flex-start'
                            }}
                            onClick={() =>
                                column.sortable && props.onSort(column.accessorKey as string)
                            }
                            role="columnheader"
                            aria-sort={
                                isColumnActive()
                                    ? props.sortOrder === 'asc'
                                        ? 'ascending'
                                        : 'descending'
                                    : 'none'
                            }
                        >
                            <span class="ui-table-grid-th-text">{column.header}</span>
                            <Show when={column.sortable}>
                                <SortIcon active={isColumnActive()} order={props.sortOrder} />
                            </Show>

                            <Show when={column.resizable !== false}>
                                <div
                                    class="ui-table-column-resizer"
                                    onPointerDown={event => handleResizeStart(column, event)}
                                    onClick={event => event.stopPropagation()}
                                />
                            </Show>
                        </div>
                    );
                }}
            </For>
        </div>
    );
}
