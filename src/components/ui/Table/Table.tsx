import {
    createSignal,
    createMemo,
    createEffect,
    onMount,
    onCleanup,
    For,
    Show,
    splitProps
} from 'solid-js';
import { cn } from '../../../lib/utils';
import { TableHeader } from './TableHeader';
import { TableRow } from './TableRow';
import { EmptyState } from './EmptyState';
import { useTableVirtualization } from './hooks/useTableVirtualization';
import { useTableNavigation } from './hooks/useTableNavigation';
import type { TableProps, SortOrder } from './types';
import './table.css';

/**
 * High-performance, virtualized data table component for Solid.js.
 * Implements the ARIA Grid pattern for accessibility and integrates with the
 * core input system for keyboard navigation.
 *
 * @template T - The record type for the table rows.
 * @param {TableProps<T>} props - Configuration properties for the table.
 * @returns {JSX.Element} The rendered virtualized table.
 *
 * @example
 * <Table
 *   data={items()}
 *   columns={[
 *     { header: 'Name', accessorKey: 'name' },
 *     { header: 'Size', accessorKey: 'size', align: 'right' }
 *   ]}
 *   height="500px"
 * />
 */
export function Table<T extends Record<string, unknown>>(props: TableProps<T>) {
    const [local] = splitProps(props, [
        'data',
        'columns',
        'rowHeight',
        'stickyHeader',
        'sortKey',
        'sortOrder',
        'selectedIds',
        'onSort',
        'onRowClick',
        'onRowDoubleClick',
        'onScroll',
        'onRowMount',
        'keyField',
        'class',
        'height',
        'label',
        'emptyMessage',
        'emptyDescription',
        'emptyIcon',
        'onVisibleItemsChange'
    ]);

    let gridContainer: HTMLDivElement | undefined;

    const [scrollTop, setScrollTop] = createSignal(0);
    const [containerHeight, setContainerHeight] = createSignal(0);
    const [focusedIndex, setFocusedIndex] = createSignal(-1);

    const rowHeight = () => local.rowHeight ?? 32;
    const HEADER_HEIGHT = 32;
    const stickyHeader = () => local.stickyHeader ?? true;
    const keyField = () => local.keyField ?? ('id' as keyof T);
    const selectedIds = () => local.selectedIds ?? [];

    const visibleColumns = createMemo(() => local.columns.filter(col => !col.hidden));
    const dataLength = createMemo(() => local.data.length);

    onMount(() => {
        if (!gridContainer) return;

        const observer = new ResizeObserver(entries => {
            const entry = entries[0];
            if (entry) {
                setContainerHeight(entry.contentRect.height);
            }
        });

        observer.observe(gridContainer);
        setContainerHeight(gridContainer.clientHeight);

        onCleanup(() => observer.disconnect());
    });

    const { visibleRange, totalHeight } = useTableVirtualization({
        dataLength,
        rowHeight,
        scrollTop,
        containerHeight,
        headerHeight: HEADER_HEIGHT
    });

    createEffect(() => {
        const range = visibleRange();
        const callback = local.onVisibleItemsChange;

        if (!callback || local.data.length === 0) return;

        const items = local.data.slice(range.start, range.end);
        const timer = setTimeout(() => {
            callback(items);
        }, 150);

        onCleanup(() => clearTimeout(timer));
    });

    const scrollToIndex = (index: number) => {
        const currentScrollTop = scrollTop();
        const currentContainerHeight = containerHeight();
        const currentRowHeight = rowHeight();

        const itemTop = index * currentRowHeight;
        const itemBottom = itemTop + currentRowHeight;

        if (itemTop < currentScrollTop + HEADER_HEIGHT) {
            gridContainer?.scrollTo({ top: itemTop - HEADER_HEIGHT });
        } else if (itemBottom > currentScrollTop + currentContainerHeight) {
            gridContainer?.scrollTo({ top: itemBottom - currentContainerHeight + HEADER_HEIGHT });
        }
    };

    useTableNavigation({
        data: () => local.data,
        focusedIndex,
        setFocusedIndex,
        onRowClick: () => local.onRowClick,
        onRowDoubleClick: () => local.onRowDoubleClick,
        scrollToIndex
    });

    const handleScroll = (e: Event) => {
        const target = e.currentTarget as HTMLDivElement;
        setScrollTop(target.scrollTop);
        local.onScroll?.(e);
    };

    const handleSort = (key: string) => {
        if (!local.onSort) return;

        let nextOrder: SortOrder = 'asc';
        if (local.sortKey === key) {
            if (local.sortOrder === 'asc') nextOrder = 'desc';
            else if (local.sortOrder === 'desc') nextOrder = null;
        }

        local.onSort(key, nextOrder);
    };

    return (
        <div
            ref={gridContainer}
            class={cn('ui-table-grid-container', local.class)}
            style={{
                height: typeof local.height === 'number' ? `${local.height}px` : local.height
            }}
            onScroll={handleScroll}
            role="grid"
            aria-label={local.label || 'Data Table'}
            aria-rowcount={local.data.length}
            aria-colcount={visibleColumns().length}
            aria-multiselectable="true"
            tabindex="0"
        >
            <div
                class="ui-table-grid-track"
                style={{ height: `${totalHeight() + HEADER_HEIGHT}px` }}
                role="presentation"
            >
                <TableHeader
                    columns={visibleColumns()}
                    sortKey={local.sortKey}
                    sortOrder={local.sortOrder}
                    sticky={stickyHeader()}
                    onSort={handleSort}
                />

                <Show
                    when={local.data.length > 0}
                    fallback={
                        <EmptyState
                            icon={local.emptyIcon}
                            message={local.emptyMessage}
                            description={local.emptyDescription}
                        />
                    }
                >
                    <For each={local.data.slice(visibleRange().start, visibleRange().end)}>
                        {(item, index) => {
                            const realIndex = createMemo(() => visibleRange().start + index());
                            const id = item[keyField()] as string | number;
                            const isSelected = createMemo(() => selectedIds().includes(id));
                            const isFocused = createMemo(() => focusedIndex() === realIndex());

                            return (
                                <TableRow
                                    item={item}
                                    realIndex={realIndex()}
                                    isSelected={isSelected()}
                                    isFocused={isFocused()}
                                    rowHeight={rowHeight()}
                                    headerHeight={HEADER_HEIGHT}
                                    columns={visibleColumns()}
                                    onMount={local.onRowMount}
                                    onClick={e => {
                                        setFocusedIndex(realIndex());
                                        local.onRowClick?.(
                                            item,
                                            e.ctrlKey || e.metaKey,
                                            e.shiftKey
                                        );
                                        gridContainer?.focus({ preventScroll: true });
                                    }}
                                    onDblClick={() => local.onRowDoubleClick?.(item)}
                                />
                            );
                        }}
                    </For>
                </Show>
            </div>
        </div>
    );
}
