import { For } from 'solid-js';
import { cn } from '../../../lib/utils';
import type { Column } from './types';

interface TableRowProps<T extends Record<string, unknown>> {
    item: T;
    realIndex: number;
    isSelected: boolean;
    isFocused: boolean;
    rowHeight: number;
    headerHeight: number;
    columns: Column<T>[];
    onClick: (e: MouseEvent) => void;
    onDblClick: () => void;
    onMount?: (el: HTMLElement, item: T) => void;
}

/**
 * Renders a single virtualized row within the table grid.
 * Handles row-level positioning, selection/focus states, and cell rendering.
 *
 * @template T - The record type for the table row data.
 * @param {TableRowProps<T>} props - Row configuration and data.
 */
export function TableRow<T extends Record<string, unknown>>(props: TableRowProps<T>) {
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
            onClick={e => props.onClick(e)}
            onDblClick={() => props.onDblClick()}
            role="row"
            aria-rowindex={props.realIndex + 2} // +1 for index and +1 for header
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
                                : (props.item[column.accessorKey as keyof T] as never)}
                        </div>
                    </div>
                )}
            </For>
        </div>
    );
}
