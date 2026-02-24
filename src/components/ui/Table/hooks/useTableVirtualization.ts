import { createMemo, Accessor } from 'solid-js';

/**
 * Configuration properties for the useTableVirtualization hook.
 */
interface UseTableVirtualizationProps {
    /** Reactive accessor for the total number of items in the dataset */
    dataLength: Accessor<number>;
    /** Reactive accessor for the fixed height of each row in pixels */
    rowHeight: Accessor<number>;
    /** Reactive accessor for the current vertical scroll position of the container */
    scrollTop: Accessor<number>;
    /** Reactive accessor for the visible height of the scrollable container */
    containerHeight: Accessor<number>;
    /** Fixed height of the table header to correct offset calculations */
    headerHeight: number;
    /** Number of additional rows to render above and below the viewport to prevent flickering */
    overscan?: number;
}

/**
 * Custom hook to calculate the range of visible rows and total content height for virtualization.
 *
 * This hook enables the "windowing" technique by determining exactly which data indices
 * need to be rendered based on the current scroll state. This significantly improves
 * performance for large lists by keeping the DOM size constant.
 *
 * @param {UseTableVirtualizationProps} props - Virtualization state accessors and configuration.
 * @returns {Object} Accessors for the current visible range and total scrollable height.
 */
export function useTableVirtualization(props: UseTableVirtualizationProps) {
    /**
     * Memoized calculation of the start and end indices of visible items.
     * Recalculates automatically when scroll position or container dimensions change.
     */
    const visibleRange = createMemo(() => {
        const overscanCount = props.overscan ?? 5;
        const currentScrollTop = props.scrollTop();
        const currentContainerHeight = props.containerHeight();
        const currentItemHeight = props.rowHeight();

        /** Calculate the first index that intersects with the top of the viewport */
        const startIndex = Math.max(
            0,
            Math.floor((currentScrollTop - props.headerHeight) / currentItemHeight) - overscanCount
        );

        /** Determine how many rows can fit entirely or partially within the container height */
        const visibleItemCount = Math.ceil(currentContainerHeight / currentItemHeight);

        /** Calculate the last index to render, ensuring it doesn't exceed data boundaries */
        const endIndex = Math.min(
            props.dataLength(),
            startIndex + visibleItemCount + overscanCount * 2
        );

        return { start: startIndex, end: endIndex };
    });

    /** The total theoretical height of the scrollable area based on dataset size */
    const totalHeight = createMemo(() => props.dataLength() * props.rowHeight());

    return { visibleRange, totalHeight };
}
