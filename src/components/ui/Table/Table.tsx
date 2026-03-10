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
import { ContextMenu, type ContextMenuItem } from '../ContextMenu';
import { Checkbox } from '../Checkbox';
import type { TableProps, SortOrder } from './types';
import { scheduler } from '../../../core/utils/scheduler';
import './table.css';

/** Constant representing the height of the table header in pixels */
const TABLE_HEADER_HEIGHT = 32;

/**
 * High-performance, virtualized data table component for Solid.js.
 *
 * Implements the ARIA Grid pattern for accessibility and integrates with the
 * core input system for keyboard navigation. Features include:
 * - Windowed (virtualized) rendering for large datasets.
 * - Dynamic column resizing and visibility toggling.
 * - Coordinated selection and focus management.
 * - Keyboard navigation via standard viewport commands.
 * - Persistence-ready configuration callbacks.
 *
 * @template T - The record type for the table rows.
 * @param {TableProps<T>} props - Configuration properties for the table.
 * @returns {JSX.Element} The rendered virtualized table component.
 *
 * @example
 * <Table
 *   data={items()}
 *   columns={[
 *     { header: 'Name', accessorKey: 'filename', resizable: true },
 *     { header: 'Size', accessorKey: 'size', align: 'right', width: 100 }
 *   ]}
 *   height="500px"
 * />
 */
export function Table<T>(props: TableProps<T>) {
    /** Separated props to maintain reactivity for non-extracted properties */
    const [local] = splitProps(props, [
        'data',
        'columns',
        'rowHeight',
        'stickyHeader',
        'sortKey',
        'sortOrder',
        'selectedIds',
        'onSort',
        'onColumnResize',
        'onColumnVisibilityChange',
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

    /** Reference to the main scrollable grid container */
    let gridContainer: HTMLDivElement | undefined;

    /** Current scroll top position of the grid container */
    const [scrollTop, setScrollTop] = createSignal(0);
    /** Current measured height of the visible grid viewport */
    const [containerHeight, setContainerHeight] = createSignal(0);
    /** Index of the currently focused row in the dataset */
    const [focusedIndex, setFocusedIndex] = createSignal(-1);
    /** State for the column visibility context menu */
    const [visibilityMenu, setVisibilityMenu] = createSignal({ isOpen: false, xPos: 0, yPos: 0 });

    /** Resolved height per row, defaulting to standard height if not provided */
    const rowHeight = createMemo(() => local.rowHeight ?? 32);
    /** Whether the header stays fixed at the top while scrolling */
    const isHeaderSticky = createMemo(() => local.stickyHeader ?? true);
    /** Property key used to uniquely identify each row item */
    const itemKeyField = createMemo(() => local.keyField ?? ('id' as keyof T));
    /** Set of identifiers for rows currently marked as selected */
    const activeSelectedIds = createMemo(() => local.selectedIds ?? []);

    /** Memoized list of columns that are currently marked as visible */
    const visibleColumns = createMemo(() => local.columns.filter(column => !column.hidden));
    /** Total number of items in the provided data array */
    const currentDataLength = createMemo(() => local.data.length);

    /** Initialize resize observer for dynamic container height tracking */
    onMount(() => {
        if (!gridContainer) {
            return;
        }

        const resizeObserver = new ResizeObserver(entries => {
            const entry = entries[0];
            if (entry) {
                setContainerHeight(entry.contentRect.height);
            }
        });

        resizeObserver.observe(gridContainer);
        setContainerHeight(gridContainer.clientHeight);

        /** Cleanup the observer on component unmount */
        onCleanup(() => resizeObserver.disconnect());
    });

    /** Instantiate virtualization range logic based on scroll and height states */
    const { visibleRange, totalHeight } = useTableVirtualization({
        dataLength: currentDataLength,
        rowHeight,
        scrollTop,
        containerHeight,
        headerHeight: TABLE_HEADER_HEIGHT
    });

    /** Detect and announce visible item changes for external consumers (e.g. lazy loading) */
    createEffect(() => {
        const range = visibleRange();
        const notifyVisibleItems = local.onVisibleItemsChange;

        if (!notifyVisibleItems || local.data.length === 0) {
            return;
        }

        const currentlyVisibleItems = local.data.slice(range.start, range.end);
        const debounceTimer = setTimeout(() => {
            notifyVisibleItems(currentlyVisibleItems);
        }, 150);

        onCleanup(() => clearTimeout(debounceTimer));
    });

    /**
     * Scrolls the container to ensure a specific item index is visible.
     * @param {number} targetIndex - The data index to scroll toward.
     */
    const handleScrollToIndex = (targetIndex: number) => {
        const topPosition = scrollTop();
        const viewportHeight = containerHeight();
        const heightPerRow = rowHeight();

        const itemTopBoundary = targetIndex * heightPerRow;
        const itemBottomBoundary = itemTopBoundary + heightPerRow;

        if (itemTopBoundary < topPosition + TABLE_HEADER_HEIGHT) {
            gridContainer?.scrollTo({ top: itemTopBoundary - TABLE_HEADER_HEIGHT });
        } else if (itemBottomBoundary > topPosition + viewportHeight) {
            gridContainer?.scrollTo({
                top: itemBottomBoundary - viewportHeight + TABLE_HEADER_HEIGHT
            });
        }
    };

    /** Hook the table up to the application's central input/command system */
    useTableNavigation({
        data: () => local.data,
        focusedIndex,
        setFocusedIndex,
        onRowClick: () => local.onRowClick,
        onRowDoubleClick: () => local.onRowDoubleClick,
        scrollToIndex: handleScrollToIndex
    });

    /** Handles the scroll event to sync reactive scroll position */
    let isScrollScheduled = false;

    const handleContainerScroll = (event: Event) => {
        const scrollableTarget = event.currentTarget as HTMLDivElement;

        if (!isScrollScheduled) {
            isScrollScheduled = true;
            scheduler.schedule(() => {
                setScrollTop(scrollableTarget.scrollTop);
                isScrollScheduled = false;
            });
        }

        local.onScroll?.(event);
    };

    /**
     * Calculates the next sort state when a header column is triggered.
     * @param {string} columnKey - The key associated with the sorted column.
     */
    const handleColumnSortTrigger = (columnKey: string) => {
        if (!local.onSort) {
            return;
        }

        let nextSortOrder: SortOrder = 'asc';
        if (local.sortKey === columnKey) {
            if (local.sortOrder === 'asc') {
                nextSortOrder = 'desc';
            } else if (local.sortOrder === 'desc') {
                nextSortOrder = null;
            }
        }

        local.onSort(columnKey, nextSortOrder);
    };

    /** Builds the item list for the column visibility context menu */
    const menuItems = createMemo<ContextMenuItem[]>(() => {
        return local.columns
            .filter(column => column.toggleable !== false)
            .map(column => ({
                type: 'custom',
                content: (
                    <div class="ui-table-grid-menu-item">
                        <Checkbox
                            label={
                                typeof column.header === 'string'
                                    ? column.header
                                    : String(column.accessorKey)
                            }
                            checked={!column.hidden}
                            onCheckedChange={checked => {
                                local.onColumnVisibilityChange?.(
                                    column.accessorKey as string,
                                    checked
                                );
                            }}
                            size="md"
                        />
                    </div>
                )
            }));
    });

    /** Slice of data currently prioritized for rendering based on visible range */
    const renderedSliceOfData = createMemo(() => {
        const { start, end } = visibleRange();
        return local.data.slice(start, end);
    });

    return (
        <div
            ref={gridContainer}
            class={cn('ui-table-grid-container', local.class)}
            style={{
                height: typeof local.height === 'number' ? `${local.height}px` : local.height
            }}
            onScroll={handleContainerScroll}
            role="grid"
            aria-label={local.label || 'Data Table'}
            aria-rowcount={local.data.length}
            aria-colcount={visibleColumns().length}
            aria-multiselectable="true"
            tabindex="0"
        >
            <div
                class="ui-table-grid-track"
                style={{ height: `${totalHeight() + TABLE_HEADER_HEIGHT}px` }}
                role="presentation"
            >
                <TableHeader
                    columns={visibleColumns()}
                    sortKey={local.sortKey}
                    sortOrder={local.sortOrder}
                    sticky={isHeaderSticky()}
                    onSort={handleColumnSortTrigger}
                    onColumnResize={local.onColumnResize}
                    onHeaderContextMenu={event => {
                        event.preventDefault();
                        setVisibilityMenu({
                            isOpen: true,
                            xPos: event.clientX,
                            yPos: event.clientY
                        });
                    }}
                />

                <ContextMenu
                    coordinateX={visibilityMenu().xPos}
                    coordinateY={visibilityMenu().yPos}
                    isOpen={visibilityMenu().isOpen}
                    items={menuItems()}
                    onClose={() => setVisibilityMenu({ ...visibilityMenu(), isOpen: false })}
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
                    <For each={renderedSliceOfData()}>
                        {(rowItem, indexAccessor) => {
                            /** Offset index representing the actual index in the full data array */
                            const resolvedGlobalIndex = createMemo(
                                () => visibleRange().start + indexAccessor()
                            );
                            /** Unique identifier for the current row */
                            const rowIdentifier = String(rowItem[itemKeyField()]);
                            /** Whether the current row is marked as selected */
                            const isRowSelected = createMemo(() =>
                                activeSelectedIds().includes(rowIdentifier)
                            );
                            /** Whether the current row currently holds keyboard focus */
                            const isRowFocused = createMemo(
                                () => focusedIndex() === resolvedGlobalIndex()
                            );

                            return (
                                <TableRow
                                    item={rowItem}
                                    realIndex={resolvedGlobalIndex()}
                                    isSelected={isRowSelected()}
                                    isFocused={isRowFocused()}
                                    rowHeight={rowHeight()}
                                    headerHeight={TABLE_HEADER_HEIGHT}
                                    columns={visibleColumns()}
                                    onMount={local.onRowMount}
                                    onClick={event => {
                                        setFocusedIndex(resolvedGlobalIndex());
                                        local.onRowClick?.(
                                            rowItem,
                                            event.ctrlKey || event.metaKey,
                                            event.shiftKey
                                        );
                                        gridContainer?.focus({ preventScroll: true });
                                    }}
                                    onDblClick={() => local.onRowDoubleClick?.(rowItem)}
                                />
                            );
                        }}
                    </For>
                </Show>
            </div>
        </div>
    );
}
