import { createSignal, onMount, onCleanup, For, createMemo, Show } from 'solid-js';
import { AssetCard } from './AssetCard';
import { EmptyState } from './EmptyState';
import { type AssetItem } from '../../../types';
import {
    useLibrary,
    useAssetCardActions,
    useVirtualViewport,
    toLayoutItems,
    useGridKeyboardNav,
    useSelection
} from '../../../core/hooks';
import { scheduler } from '../../../core/utils/scheduler';
import './viewport.css';

interface VirtualMasonryProps {
    items: AssetItem[];
    mode?: 'masonry-v' | 'masonry-h';
    gap?: number;
    buffer?: number;
}

/**
 * VirtualMasonry - Worker-based Virtualized Masonry Layout
 *
 * Uses a Web Worker for layout calculations and Spatial Grid for O(1) visibility queries.
 */
export function VirtualMasonry(props: VirtualMasonryProps) {
    const library = useLibrary();
    const actions = useAssetCardActions();
    const selection = useSelection();

    const [scrollContainer, setScrollContainer] = createSignal<HTMLDivElement>();
    const [containerHeight, setContainerHeight] = createSignal(0);

    // Convert items to Worker-friendly format (minimal data)
    const layoutItems = createMemo(() => toLayoutItems(props.items));

    // Connect to the layout Worker with the specified mode
    const layoutMode = () => props.mode || 'masonry-v';
    const viewport = useVirtualViewport(layoutMode, () => layoutItems(), {
        get gap() {
            return props.gap;
        },
        get buffer() {
            return props.buffer;
        }
    });

    const itemsById = createMemo(() => {
        const map = new Map<number, AssetItem>();
        props.items.forEach(item => map.set(item.id, item));
        return map;
    });

    const keyboardNav = useGridKeyboardNav({
        visibleItems: viewport.visibleItems,
        allItems: () => props.items,
        containerHeight,
        scrollContainer,
        onSelect: (itemId: number, modifiers: { multi: boolean; shift: boolean }) =>
            actions.handleSelect(itemId, modifiers),
        onOpen: itemId => actions.handleOpen(itemId),
        isSelected: actions.isSelected,
        getSelectedIds: actions.getSelectedIds,
        getItemRect: itemId => viewport.getItemPosition(itemId)
    });

    const handleSelectWithFocus = (
        itemId: number,
        modifiers: { multi: boolean; shift: boolean }
    ) => {
        keyboardNav.syncFocusWithClick(itemId);
        actions.handleSelect(itemId, modifiers);
    };

    const getItemInfo = (itemId: number) => {
        const item = itemsById().get(itemId);
        if (!item) return undefined;
        return {
            path: item.path,
            thumbnail_path: item.thumbnail_path
        };
    };

    let lastReportedWidth = 0;

    onMount(() => {
        const element = scrollContainer();
        if (!element) return;

        const observer = new ResizeObserver(entries => {
            scheduler.schedule(() => {
                const entry = entries[0];
                if (!entry) return;

                const width = entry.contentRect.width;
                const height = entry.contentRect.height;

                setContainerHeight(height);

                if (width > 0 && Math.abs(width - lastReportedWidth) > 5) {
                    lastReportedWidth = width;
                    viewport.handleResize(width);
                }
            });
        });

        observer.observe(element);

        const initialRect = element.getBoundingClientRect();
        if (initialRect.width > 0) {
            setContainerHeight(initialRect.height);
            lastReportedWidth = initialRect.width;
            viewport.handleResize(initialRect.width);
        } else {
            const estimatedWidth = window.innerWidth - 80;
            if (estimatedWidth > 0) {
                lastReportedWidth = estimatedWidth;
                viewport.handleResize(estimatedWidth);
            }
        }

        const handleScroll = () => {
            const containerElement = scrollContainer();
            if (!containerElement) return;

            viewport.handleScroll(containerElement.scrollTop, containerHeight());

            const { scrollTop, scrollHeight, clientHeight } = containerElement;
            if (scrollTop + clientHeight >= scrollHeight - 500) {
                library.loadMore();
            }
        };

        element.addEventListener('scroll', handleScroll, { passive: true });
        viewport.handleScroll(0, containerHeight());

        onCleanup(() => {
            observer.disconnect();
            element.removeEventListener('scroll', handleScroll);
        });
    });

    return (
        <div
            ref={setScrollContainer}
            class="virtual-scroll-container"
            role="grid"
            aria-label="Asset gallery - masonry layout"
            tabIndex={0}
        >
            <Show
                when={props.items.length > 0}
                fallback={
                    <EmptyState
                        title="No assets found"
                        description="Try adjusting your filters or add assets to your library."
                    />
                }
            >
                <div
                    class="virtual-track"
                    role="rowgroup"
                    style={{
                        height: `${viewport.totalHeight()}px`,
                        position: 'relative'
                    }}
                >
                    <For each={viewport.visibleItems()}>
                        {position => {
                            const item = itemsById().get(position.id);
                            if (!item) return null;

                            const isFocused = () => keyboardNav.focusedId() === item.id;

                            return (
                                <AssetCard
                                    id={item.id}
                                    filename={item.filename}
                                    path={item.path}
                                    thumbnailPath={item.thumbnail_path}
                                    width={item.width}
                                    height={item.height}
                                    isSelected={selection.isItemSelected(item.id)}
                                    isFocused={isFocused()}
                                    style={{
                                        position: 'absolute',
                                        transform: `translate3d(${position.x}px, ${position.y}px, 0)`,
                                        width: `${position.width}px`,
                                        height: `${position.height}px`
                                    }}
                                    onSelect={handleSelectWithFocus}
                                    onOpen={actions.handleOpen}
                                    getSelectedIds={actions.getSelectedIds}
                                    getItemInfo={getItemInfo}
                                />
                            );
                        }}
                    </For>
                </div>
            </Show>
        </div>
    );
}
