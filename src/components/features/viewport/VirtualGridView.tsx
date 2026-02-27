import { Component, createSignal, onMount, onCleanup, For, createMemo, Show } from 'solid-js';
import { AssetCard } from './AssetCard';
import { EmptyState } from './EmptyState';
import {
    useLibrary,
    useAssetCardActions,
    useVirtualViewport,
    useGridKeyboardNav,
    useSelection
} from '../../../core/hooks';
import { scheduler } from '../../../core/utils/scheduler';
import type { LayoutItemInput } from '../../../core/viewport';
import './grid-view.css';

/**
 * VirtualGridView - Worker-based Virtualized Grid Layout
 *
 * Uses a Web Worker for layout calculations and Spatial Grid for O(1) visibility queries.
 * Grid layout uses uniform square cells, so aspectRatio is always 1.
 */
export const VirtualGridView: Component = () => {
    const library = useLibrary();
    const actions = useAssetCardActions();
    const selection = useSelection();

    const [scrollContainer, setScrollContainer] = createSignal<HTMLDivElement>();
    const [containerHeight, setContainerHeight] = createSignal(0);

    // For grid, all items have aspectRatio = 1 (square cells)
    const layoutItems = createMemo((): LayoutItemInput[] =>
        library.items.map(item => ({
            id: item.id,
            aspectRatio: 1
        }))
    );

    // Connect to the layout Worker in grid mode
    // eslint-disable-next-line solid/reactivity
    const viewport = useVirtualViewport('grid', layoutItems);

    const itemsById = createMemo(() => {
        const map = new Map<number, (typeof library.items)[0]>();
        library.items.forEach(item => map.set(item.id, item));
        return map;
    });

    const keyboardNav = useGridKeyboardNav({
        visibleItems: viewport.visibleItems,
        allItems: () => library.items,
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
            class="grid-view-container"
            role="grid"
            aria-label="Image gallery - grid layout"
            tabIndex={0}
        >
            <Show
                when={library.items.length > 0}
                fallback={
                    <EmptyState
                        title="No images found"
                        description="Try adjusting your filters or add images to your library."
                    />
                }
            >
                <div
                    class="grid-view-track"
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
                                        top: 0,
                                        left: 0,
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
};
