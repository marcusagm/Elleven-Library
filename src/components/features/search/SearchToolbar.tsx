import { Button } from '../../ui';
import { Component, createSignal, Show, For, createMemo } from 'solid-js';
import { Search, SlidersHorizontal, Funnel, X, Sparkles } from 'lucide-solid';
import { useFilters, useMetadata, useNotification } from '../../../core/hooks';
import { SearchGroup } from '../../../core/store/filter';
import { Input } from '../../ui/Input';
import { Popover } from '../../ui/Popover';
import { AdvancedSearchModal } from './AdvancedSearchModal';
import { createConditionalScope, useCommands, shortcutStore } from '../../../core/input';
import { formatShortcutForDisplay } from '../../../core/input/normalizer';
import { cn } from '../../../lib/utils';
import './search-toolbar.css';

export const SearchToolbar: Component = () => {
    const filters = useFilters();
    const metadata = useMetadata();
    const notification = useNotification();
    const [isModalOpen, setIsModalOpen] = createSignal(false);
    const [isFocused, setIsFocused] = createSignal(false);
    let inputRef: HTMLInputElement | undefined;

    // Input System - Scope only active when searching or focused
    createConditionalScope('search', () => !!filters.searchQuery || isFocused());

    const focusSearchShortcut = createMemo(() =>
        shortcutStore.getByNameAndScope('Focus Search', 'global')
    );

    const shortcutLabel = createMemo(() => {
        const s = focusSearchShortcut();
        if (!s) return 'Cmd+K';
        const keys = Array.isArray(s.keys) ? s.keys[0] : s.keys;
        return formatShortcutForDisplay(keys);
    });

    useCommands({
        'app:focus-search': () => {
            // Check payload if needed, but command usually implies action
            // e is Payload, need event?
            // The handler in useShortcuts had 'e'. which is Event | null.
            // Dispatcher emits payload.
            // However, useCommand handler is (payload).
            // Payload doesn't have preventDefault.
            // But dispatcher handles preventDefault based on shortcut definition!
            // Global Focus Search has preventDefault: true (default).
            inputRef?.focus();
        },
        'search:close': () => {
            if (
                activeFiltersList().length > 0 ||
                filters.searchQuery ||
                document.activeElement === inputRef
            ) {
                // Only act if "enabled" condition was met
                // The original condition was:
                // enabled: () => !!filters.searchQuery || document.activeElement === inputRef

                if (filters.searchQuery) {
                    filters.setSearch('');
                } else if (document.activeElement === inputRef) {
                    inputRef?.blur();
                }
            }
        }
    });

    // Check if current advanced search matches a smart folder
    const currentSmartFolder = createMemo(() => {
        if (!filters.advancedSearch) return null;
        const currentJson = JSON.stringify(filters.advancedSearch);
        return metadata.smartFolders.find(sf => sf.query_json === currentJson);
    });

    // Filter helpers
    const activeFiltersList = () => {
        const list: { type: string; label: string; value: string; onRemove: () => void }[] = [];

        if (filters.selectedTags.length > 0) {
            filters.selectedTags.forEach(id => {
                const tag = metadata.tags.find(t => t.id === id);
                if (tag) {
                    list.push({
                        type: 'tag',
                        label: 'Tag',
                        value: tag.name,
                        onRemove: () => filters.toggleTag(id)
                    });
                }
            });
        }

        if (filters.selectedFolderId !== null) {
            const folder = metadata.locations.find(f => f.id === filters.selectedFolderId);
            if (folder) {
                list.push({
                    type: 'folder',
                    label: 'Folder',
                    value: folder.name,
                    onRemove: () => filters.setFolder(null)
                });
            }
        }

        if (filters.filterUntagged) {
            list.push({
                type: 'untagged',
                label: 'Filter',
                value: 'Untagged',
                onRemove: () => filters.setUntagged(false)
            });
        }

        if (filters.advancedSearch) {
            const smartFolder = currentSmartFolder();

            list.push({
                type: 'advanced',
                label: smartFolder ? 'Smart Folder' : 'Advanced',
                value: smartFolder
                    ? smartFolder.name
                    : `${filters.advancedSearch.items.length} criteria active`,
                onRemove: () => filters.setAdvancedSearch(null)
            });
        }

        return list;
    };

    return (
        <div class="search-toolbar">
            <div class="search-input-wrapper">
                <Input
                    ref={inputRef}
                    placeholder={`Search references (${shortcutLabel()})`}
                    value={filters.searchQuery}
                    onInput={e => filters.setSearch(e.currentTarget.value)}
                    onFocus={() => setIsFocused(true)}
                    onBlur={() => setIsFocused(false)}
                    leftIcon={<Search size={14} />}
                    class="search-input"
                />
                <div class="search-actions">
                    <Show when={activeFiltersList().length > 0}>
                        <Popover
                            placement="bottom-end"
                            trigger={
                                <Button
                                    variant="ghost"
                                    size="icon-xs"
                                    class="search-action-btn active"
                                    title="Active Filters"
                                >
                                    <Funnel />
                                </Button>
                            }
                        >
                            <div class="active-filters-popover">
                                <div class="active-filters-header">
                                    <h4>Active Filters</h4>
                                    <Button
                                        variant="ghost"
                                        size="xs"
                                        onClick={() => filters.clearAll()}
                                    >
                                        Clear All
                                    </Button>
                                </div>
                                <div class="active-filters-list">
                                    <For each={activeFiltersList()}>
                                        {filter => (
                                            <div class="active-filter-item">
                                                <span class="filter-label">{filter.label}:</span>
                                                <span class="filter-value">{filter.value}</span>
                                                <Button
                                                    variant="ghost"
                                                    size="icon-xs"
                                                    class="remove-filter-btn"
                                                    onClick={filter.onRemove}
                                                >
                                                    <X />
                                                </Button>
                                            </div>
                                        )}
                                    </For>
                                </div>
                            </div>
                        </Popover>
                    </Show>

                    <Button
                        variant="ghost"
                        size="icon-xs"
                        class={cn('search-action-btn', !!filters.searchFuzzy && 'active')}
                        title={filters.searchFuzzy ? 'Fuzzy Match: ON' : 'Fuzzy Match: OFF'}
                        onClick={() => filters.setSearchFuzzy(!filters.searchFuzzy)}
                    >
                        <Sparkles />
                    </Button>

                    <Button
                        variant="ghost"
                        size="icon-xs"
                        class={cn('search-action-btn', !!filters.advancedSearch && 'active')}
                        title="Advanced Search"
                        onClick={() => setIsModalOpen(true)}
                    >
                        <SlidersHorizontal />
                    </Button>
                </div>
            </div>

            <AdvancedSearchModal
                isOpen={isModalOpen()}
                onClose={() => setIsModalOpen(false)}
                isSmartFolderMode={!!currentSmartFolder()}
                initialIdentifier={currentSmartFolder()?.id}
                initialName={currentSmartFolder()?.name}
                initialQuery={filters.advancedSearch || undefined}
                onSave={(name: string, query: SearchGroup, id?: number) =>
                    metadata.saveSmartFolder(name, query, id).then(result => {
                        if (result.success) {
                            notification.success(
                                id ? 'Smart Folder Updated' : 'Smart Folder Created',
                                `Saved "${name}"`
                            );
                        } else {
                            notification.error(
                                result.error?.message || 'Failed to Save Smart Folder'
                            );
                        }
                        return result;
                    })
                }
            />
        </div>
    );
};
