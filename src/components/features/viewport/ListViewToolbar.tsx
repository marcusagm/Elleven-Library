import {
    Button,
    ButtonGroup,
    PopoverRoot,
    PopoverTrigger,
    PopoverContent,
    Select,
    Slider,
    ToggleGroup,
    ToggleGroupItem,
    Tooltip
} from '../../ui';
import { SearchToolbar } from '../search/SearchToolbar';
import { Component } from 'solid-js';
import {
    ArrowLeft,
    ArrowRight,
    LayoutGrid,
    AlignCenterVertical,
    AlignCenterHorizontal,
    List,
    SortAsc,
    SortDesc,
    ChevronDown
} from 'lucide-solid';
import { useFilters } from '../../../core/hooks';
import './list-view-toolbar.css';

/**
 * Renders the toolbar for the main list view, containing history navigation, search, sorting, and view options.
 *
 * @returns {JSX.Element} The list view toolbar container.
 *
 * @example
 * ```tsx
 * import { ListViewToolbar } from '@/components/features/viewport/ListViewToolbar';
 * <ListViewToolbar />
 * ```
 */
export const ListViewToolbar: Component = () => {
    /**
     * Filters for the list view toolbar.
     */
    const filters = useFilters();

    return (
        <div class="list-view-toolbar">
            {/* History Navigation */}
            <div class="toolbar-group">
                <ButtonGroup attached>
                    <Tooltip content="Back" placement="bottom">
                        <Button
                            variant="secondary"
                            size="icon"
                            onClick={() => filters.goBack()}
                            disabled={!filters.canGoBack}
                        >
                            <ArrowLeft size={18} />
                        </Button>
                    </Tooltip>
                    <Tooltip content="Forward" placement="bottom">
                        <Button
                            variant="secondary"
                            size="icon"
                            onClick={() => filters.goForward()}
                            disabled={!filters.canGoForward}
                        >
                            <ArrowRight size={18} />
                        </Button>
                    </Tooltip>
                </ButtonGroup>
            </div>

            {/* Search Bar */}
            <div class="toolbar-search">
                <SearchToolbar />
            </div>

            {/* Sort & View Controls */}
            <div class="toolbar-group">
                {/* Sort Popover */}
                <PopoverRoot placement="bottom-end">
                    <PopoverTrigger>
                        <Button variant="ghost" class="sort-dropdown-trigger">
                            <span>
                                Sort:{' '}
                                {{
                                    modified_at: 'Modification',
                                    added_at: 'Addition',
                                    created_at: 'Creation',
                                    filename: 'Title',
                                    format: 'Type',
                                    size: 'Size',
                                    rating: 'Rating'
                                }[filters.sortBy] || 'Date'}
                            </span>
                            <ChevronDown size={14} />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent class="toolbar-popover-sort-configuration">
                        <div class="popover-config-group">
                            <span class="popover-config-label">Property</span>
                            <Select
                                options={[
                                    { value: 'modified_at', label: 'Modification Date' },
                                    { value: 'added_at', label: 'Addition Date' },
                                    { value: 'created_at', label: 'Creation Date' },
                                    { value: 'filename', label: 'Title' },
                                    { value: 'format', label: 'File Type' },
                                    { value: 'size', label: 'File Size' },
                                    { value: 'rating', label: 'Rating' }
                                ]}
                                value={filters.sortBy}
                                onValueChange={(val: string) =>
                                    val &&
                                    filters.setSortBy(
                                        val as
                                            | 'modified_at'
                                            | 'added_at'
                                            | 'created_at'
                                            | 'filename'
                                            | 'format'
                                            | 'size'
                                            | 'rating'
                                    )
                                }
                            />
                        </div>

                        <div class="popover-config-group" style={{ 'margin-top': '16px' }}>
                            <span class="popover-config-label">Direction</span>
                            <ToggleGroup
                                type="single"
                                value={filters.sortOrder}
                                onValueChange={(val: string) =>
                                    val && filters.setSortOrder(val as 'asc' | 'desc')
                                }
                            >
                                <ToggleGroupItem value="asc" title="Ascending" style={{ flex: 1 }}>
                                    <div
                                        style={{
                                            display: 'flex',
                                            'align-items': 'center',
                                            gap: '8px'
                                        }}
                                    >
                                        <SortAsc size={14} />
                                        <span>Ascending</span>
                                    </div>
                                </ToggleGroupItem>
                                <ToggleGroupItem
                                    value="desc"
                                    title="Descending"
                                    style={{ flex: 1 }}
                                >
                                    <div
                                        style={{
                                            display: 'flex',
                                            'align-items': 'center',
                                            gap: '8px'
                                        }}
                                    >
                                        <SortDesc size={14} />
                                        <span>Descending</span>
                                    </div>
                                </ToggleGroupItem>
                            </ToggleGroup>
                        </div>
                    </PopoverContent>
                </PopoverRoot>

                <div class="toolbar-separator" />

                {/* View Layout Popover */}
                <PopoverRoot placement="bottom-end">
                    <PopoverTrigger>
                        <Tooltip content="View Options" placement="bottom">
                            <Button variant="ghost" size="icon">
                                <LayoutGrid size={18} />
                            </Button>
                        </Tooltip>
                    </PopoverTrigger>
                    <PopoverContent class="toolbar-popover-view-configuration">
                        <div class="popover-config-group">
                            <span class="popover-config-label">Layout Mode</span>
                            <ToggleGroup
                                type="single"
                                value={filters.layout}
                                onValueChange={(val: string) =>
                                    val &&
                                    filters.setLayout(
                                        val as 'grid' | 'list' | 'masonry-v' | 'masonry-h'
                                    )
                                }
                            >
                                <Tooltip content="Masonry Vertical" placement="top">
                                    <ToggleGroupItem value="masonry-v">
                                        <AlignCenterVertical size={16} />
                                    </ToggleGroupItem>
                                </Tooltip>
                                <Tooltip content="Masonry Horizontal" placement="top">
                                    <ToggleGroupItem value="masonry-h">
                                        <AlignCenterHorizontal size={16} />
                                    </ToggleGroupItem>
                                </Tooltip>
                                <Tooltip content="Grid" placement="top">
                                    <ToggleGroupItem value="grid">
                                        <LayoutGrid size={16} />
                                    </ToggleGroupItem>
                                </Tooltip>
                                <Tooltip content="List" placement="top">
                                    <ToggleGroupItem value="list">
                                        <List size={16} />
                                    </ToggleGroupItem>
                                </Tooltip>
                            </ToggleGroup>
                        </div>

                        <div class="popover-config-group" style={{ 'margin-top': '16px' }}>
                            <span class="popover-config-label">Thumbnail Size</span>
                            <div style={{ 'padding-top': '8px' }}>
                                <Slider
                                    value={filters.thumbSize || 200}
                                    minimumValue={100}
                                    maximumValue={500}
                                    showTicks={false}
                                    onValueChange={newThumbnailSize =>
                                        filters.setThumbSize(newThumbnailSize)
                                    }
                                    title="Thumbnail Size"
                                />
                            </div>
                        </div>
                    </PopoverContent>
                </PopoverRoot>
            </div>
        </div>
    );
};
