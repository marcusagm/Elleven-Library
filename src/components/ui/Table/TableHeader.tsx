import { For, Show } from 'solid-js';
import { ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import type { Column, SortOrder } from './types';

interface TableHeaderProps<T extends Record<string, unknown>> {
    columns: Column<T>[];
    sortKey: string | null | undefined;
    sortOrder: SortOrder | undefined;
    sticky: boolean;
    onSort: (key: string) => void;
}

/**
 * Properties for the SortIcon helper component.
 */
interface SortIconProps {
    /** Whether sorting is active for this column */
    active: boolean;
    /** The current sort order */
    order: SortOrder | undefined;
}

/**
 * Internal helper component to render the sorting state icon.
 * Displays up, down, or both-directions icons based on sort state.
 *
 * @param {SortIconProps} props - Sort state properties.
 */
function SortIcon(props: SortIconProps) {
    return (
        <span class="ui-table-grid-sort-icon">
            {props.active ? (
                props.order === 'asc' ? (
                    <ChevronUp size={12} />
                ) : (
                    <ChevronDown size={12} />
                )
            ) : (
                <ChevronsUpDown size={12} />
            )}
        </span>
    );
}

/**
 * Renders the interactive header row of the table.
 * Manages column widths, alignment, and sort triggers.
 *
 * @template T - The record type for the table.
 * @param {TableHeaderProps<T>} props - Header properties and callbacks.
 */
export function TableHeader<T extends Record<string, unknown>>(props: TableHeaderProps<T>) {
    return (
        <div
            class={cn('ui-table-grid-header', props.sticky && 'ui-table-grid-header-sticky')}
            role="row"
            aria-rowindex={1}
        >
            <For each={props.columns}>
                {column => (
                    <div
                        class={cn(
                            'ui-table-grid-th',
                            column.sortable && 'ui-table-grid-th-sortable',
                            props.sortKey === column.accessorKey && 'ui-table-grid-th-active'
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
                            props.sortKey === column.accessorKey
                                ? props.sortOrder === 'asc'
                                    ? 'ascending'
                                    : 'descending'
                                : 'none'
                        }
                    >
                        <span class="ui-table-grid-th-text">{column.header}</span>
                        <Show when={column.sortable}>
                            <SortIcon
                                active={props.sortKey === column.accessorKey}
                                order={props.sortOrder}
                            />
                        </Show>
                    </div>
                )}
            </For>
        </div>
    );
}
