import { Component, JSX } from 'solid-js';
import {
    Button,
    PopoverRoot,
    PopoverTrigger,
    PopoverContent,
    Select,
    ToggleGroup,
    ToggleGroupItem
} from '../../../ui';
import { ChevronDown, SortAsc, SortDesc } from 'lucide-solid';
import { useFilters } from '../../../../core/hooks';

/**
 * Component responsible for selecting sorting property and direction.
 * Uses the filters store to manage sort state.
 *
 * @returns {JSX.Element} The sort configuration popover.
 */
export const SortConfiguration: Component = (): JSX.Element => {
    /**
     * Filters store
     */
    const filters = useFilters();

    return (
        <PopoverRoot placement="bottom-end">
            <PopoverTrigger>
                <Button variant="ghost" class="sort-dropdown-trigger">
                    <span>
                        Sort:{' '}
                        {(
                            {
                                modified_at: 'Modification',
                                added_at: 'Addition',
                                created_at: 'Creation',
                                filename: 'Title',
                                format: 'Type',
                                size: 'Size',
                                rating: 'Rating'
                            } as Record<string, string>
                        )[filters.sortBy] || 'Date'}
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
                        <ToggleGroupItem value="desc" title="Descending" style={{ flex: 1 }}>
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
    );
};
