import { Component, createMemo, Show, createSignal, createEffect } from 'solid-js';
import { Table, Column } from '../../ui/Table';
import {
    useLibrary,
    useSelection,
    useViewport,
    useFilters,
    useAssetCardActions
} from '../../../core/hooks';
import { type SortField } from '../../../core/store/filter';
import { AssetItem } from '../../../types';
import { formatFileSize, formatDate } from '../../../utils/format';
import { assetDnD } from '../../../core/dnd';
import { EmptyState } from './EmptyState';
import { createConditionalScope } from '../../../core/input';
import { scheduler } from '../../../core/utils/scheduler';

const COLUMN_STORAGE_KEY = 'mundam-viewport-columns-v1';

const THUMBNAIL_SCALE_FACTOR = 5;
const THUMBNAIL_ASPECT_RATIO = 0.75;
const LIST_MIN_ROW_HEIGHT = 32;
const LIST_ROW_PADDING = 8;
const SCROLL_LOAD_MORE_THRESHOLD = 500;

const DEFAULT_COLS = {
    thumbnailPadding: 16,
    filename: 300,
    rating: 100,
    format: 80,
    size: 100,
    width: 120,
    date: 160
} as const;

interface ColumnConfig {
    width: number;
    hidden: boolean;
}

export const VirtualListView: Component = () => {
    const lib = useLibrary();
    const selection = useSelection();
    const viewport = useViewport();
    const filters = useFilters();
    const actions = useAssetCardActions();

    // Register viewport scope
    createConditionalScope('viewport', () => lib.items.length > 0);

    const getThumbUrl = (path: string | null) => {
        if (!path) return undefined;
        // Don't just take the filename! 'extensions/icon_xxx.webp' needs the full path.
        const normalizedPath = path.replace(/\\/g, '/');
        return `thumb://localhost/${normalizedPath}`;
    };

    const [columnConfigs, setColumnConfigs] = createSignal<Record<string, ColumnConfig>>(
        (() => {
            const saved = localStorage.getItem(COLUMN_STORAGE_KEY);
            try {
                return saved ? JSON.parse(saved) : {};
            } catch {
                return {};
            }
        })()
    );

    createEffect(() => {
        localStorage.setItem(COLUMN_STORAGE_KEY, JSON.stringify(columnConfigs()));
    });

    const updateColumnConfig = (key: string, updates: Partial<ColumnConfig>) => {
        setColumnConfigs(prev => {
            const current =
                prev[key] ||
                ({
                    width:
                        columns().find(col => col.accessorKey === key)?.width ||
                        DEFAULT_COLS.filename,
                    hidden: false
                } as ColumnConfig);
            return {
                ...prev,
                [key]: { ...current, ...updates }
            };
        });
    };

    const getColumnWidth = (key: string, defaultWidth: number) => {
        return columnConfigs()[key]?.width ?? defaultWidth;
    };

    const isColumnHidden = (key: string, defaultHidden: boolean) => {
        return columnConfigs()[key]?.hidden ?? defaultHidden;
    };

    const listThumbWidth = createMemo(() => Math.floor(filters.thumbSize / THUMBNAIL_SCALE_FACTOR));
    const listThumbHeight = createMemo(() => Math.floor(listThumbWidth() * THUMBNAIL_ASPECT_RATIO));
    const rowHeight = createMemo(() =>
        Math.max(LIST_MIN_ROW_HEIGHT, listThumbHeight() + LIST_ROW_PADDING)
    );

    const columns = createMemo<Column<AssetItem>[]>(() => [
        {
            header: '',
            accessorKey: 'thumbnail_path',
            width: getColumnWidth(
                'thumbnail_path',
                listThumbWidth() + DEFAULT_COLS.thumbnailPadding
            ),
            align: 'center',
            resizable: true,
            toggleable: false,
            cell: item => (
                <div
                    class="list-view-thumbnail-container"
                    style={{
                        width: `${listThumbWidth()}px`,
                        height: `${listThumbHeight()}px`
                    }}
                >
                    {item.thumbnail_path && (
                        <img
                            src={getThumbUrl(item.thumbnail_path)}
                            alt=""
                            draggable={false}
                            class="list-view-thumbnail"
                        />
                    )}
                </div>
            )
        },
        {
            header: 'Name',
            accessorKey: 'filename',
            sortable: true,
            resizable: true,
            toggleable: false,
            width: getColumnWidth('filename', DEFAULT_COLS.filename)
        },
        {
            header: 'Rating',
            accessorKey: 'rating',
            sortable: true,
            resizable: true,
            width: getColumnWidth('rating', DEFAULT_COLS.rating),
            hidden: isColumnHidden('rating', false),
            align: 'center',
            cell: item => (
                <span class="list-view-rating-cell">
                    {item.rating ? '★'.repeat(item.rating) : '-'}
                </span>
            )
        },
        {
            header: 'Type',
            accessorKey: 'format',
            sortable: true,
            resizable: true,
            width: getColumnWidth('format', DEFAULT_COLS.format),
            hidden: isColumnHidden('format', false),
            align: 'center',
            cell: item => (
                <span class="list-view-type-cell">{item.format?.toUpperCase() || 'N/A'}</span>
            )
        },
        {
            header: 'Size',
            accessorKey: 'size',
            sortable: true,
            resizable: true,
            width: getColumnWidth('size', DEFAULT_COLS.size),
            hidden: isColumnHidden('size', false),
            align: 'right',
            cell: item => <span>{formatFileSize(item.size)}</span>
        },
        {
            header: 'Dimensions',
            accessorKey: 'width',
            resizable: true,
            width: getColumnWidth('width', DEFAULT_COLS.width),
            hidden: isColumnHidden('width', false),
            align: 'center',
            cell: item => (
                <span>{item.width && item.height ? `${item.width} × ${item.height}` : '-'}</span>
            )
        },
        {
            header: 'Created',
            accessorKey: 'created_at',
            sortable: true,
            resizable: true,
            width: getColumnWidth('created_at', DEFAULT_COLS.date),
            hidden: isColumnHidden('created_at', true),
            cell: item => <span class="list-view-date-cell">{formatDate(item.created_at)}</span>
        },
        {
            header: 'Modified',
            accessorKey: 'modified_at',
            sortable: true,
            resizable: true,
            width: getColumnWidth('modified_at', DEFAULT_COLS.date),
            hidden: isColumnHidden('modified_at', true),
            cell: item => <span class="list-view-date-cell">{formatDate(item.modified_at)}</span>
        },
        {
            header: 'Added',
            accessorKey: 'added_at',
            sortable: true,
            resizable: true,
            width: getColumnWidth('added_at', DEFAULT_COLS.date),
            hidden: isColumnHidden('added_at', true),
            cell: item => <span class="list-view-date-cell">{formatDate(item.added_at)}</span>
        }
    ]);

    const handleSort = (key: string) => {
        if (filters.sortBy === key) {
            const nextOrder = filters.sortOrder === 'asc' ? 'desc' : 'asc';
            filters.setSortOrder(nextOrder);
        } else {
            filters.setSortBy(key as SortField);
            filters.setSortOrder('desc');
        }
    };

    let isScrollScheduled = false;

    const handleScroll = (event: Event) => {
        const target = event.currentTarget as HTMLDivElement;

        if (!isScrollScheduled) {
            isScrollScheduled = true;
            scheduler.schedule(() => {
                if (
                    target.scrollTop + target.clientHeight >=
                    target.scrollHeight - SCROLL_LOAD_MORE_THRESHOLD
                ) {
                    lib.loadMore();
                }
                isScrollScheduled = false;
            });
        }
    };

    return (
        <div class="virtual-list-view">
            <Show when={lib.items.length > 0} fallback={<EmptyState />}>
                <Table
                    data={lib.items}
                    columns={columns()}
                    rowHeight={rowHeight()}
                    height="100%"
                    sortKey={filters.sortBy}
                    sortOrder={filters.sortOrder}
                    selectedIds={selection.selectedIds}
                    onSort={handleSort}
                    onColumnResize={(key, width) => updateColumnConfig(key, { width })}
                    onColumnVisibilityChange={(key, visible) =>
                        updateColumnConfig(key, { hidden: !visible })
                    }
                    onRowClick={(item, multi, shift) => {
                        actions.handleSelect(item.id, { multi, shift });
                    }}
                    onRowDoubleClick={item => {
                        viewport.openItem(item.id.toString());
                    }}
                    onScroll={handleScroll}
                    onRowMount={(element, item) => {
                        assetDnD(element, () => ({
                            item,
                            selected: selection.isSelected(item.id),
                            selectedIds: selection.selectedIds,
                            allItems: lib.items
                        }));
                    }}
                    onVisibleItemsChange={items => {
                        const ids = items.map(item => item.id);
                        lib.setThumbnailPriority(ids);
                    }}
                />
            </Show>
        </div>
    );
};
