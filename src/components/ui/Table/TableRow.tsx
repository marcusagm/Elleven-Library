import { For, JSX } from 'solid-js';
import { cn } from '../../../lib/utils';
import type { Column } from './types';

/**
 * Properties for the TableRow component.
 */
interface TableRowProps<T> {
    /** The actual data item representing this row */
    item: T;
    /** The index of this item in the full data collection */
    realIndex: number;
    /** Whether this specific row is selected */
    isSelected: boolean;
    /** Whether this specific row is currently focused via keyboard or click */
    isFocused: boolean;
    /** Fixed height of the row in pixels */
    rowHeight: number;
    /** Total height of the table header to offset positioning */
    headerHeight: number;
    /** Set of column definitions used to render cells */
    columns: Column<T>[];
    /** Triggered when the row is clicked (single click) */
    onClick: (event: MouseEvent) => void;
    /** Triggered when the row is double-clicked */
    onDblClick: () => void;
    /** Callback triggered when the row DOM element is first mounted */
    onMount?: (element: HTMLElement, item: T) => void;
}

/**
 * Renders a single virtualized row within the table grid.
 *
 * Handles row-level positioning using absolute transforms, selection/focus states,
 * and iterates through column definitions to render individual cells.
 *
 * @template T - The record type for the table row data.
 * @param {TableRowProps<T>} props - Row configuration and data.
 * @returns {JSX.Element} The rendered table row container.
 */
export function TableRow<T>(props: TableRowProps<T>) {
    return (
        <div
            ref={element => props.onMount?.(element, props.item)}
            class={cn(
                'ui-table-grid-row',
                props.isSelected && 'ui-table-grid-row-selected',
                props.isFocused && 'ui-table-grid-row-focused'
            )}
            style={{
                height: `${props.rowHeight}px`,
                transform: `translate3d(0, ${props.headerHeight + props.realIndex * props.rowHeight}px, 0)`
            }}
            onClick={event => props.onClick(event)}
            onDblClick={() => props.onDblClick()}
            role="row"
            aria-rowindex={props.realIndex + 2} // +1 for 1-based index and +1 for header offset
            aria-selected={props.isSelected}
        >
            <For each={props.columns}>
                {column => (
                    <div
                        class="ui-table-grid-cell"
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
                        role="gridcell"
                        aria-colindex={props.columns.indexOf(column) + 1}
                    >
                        <div class="ui-table-grid-cell-content">
                            {column.cell
                                ? column.cell(props.item)
                                : (props.item[column.accessorKey as keyof T] as JSX.Element)}
                        </div>
                    </div>
                )}
            </For>
        </div>
    );
}
