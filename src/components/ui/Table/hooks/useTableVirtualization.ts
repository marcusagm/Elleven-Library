import { createMemo, Accessor } from 'solid-js';

interface UseTableVirtualizationProps {
    dataLength: Accessor<number>;
    rowHeight: Accessor<number>;
    scrollTop: Accessor<number>;
    containerHeight: Accessor<number>;
    headerHeight: number;
    overscan?: number;
}

/**
 * Custom hook to calculate visible row ranges and total scrollable height for virtualization.
 * Efficiency is maintained through memoized calculations based on scroll position and container height.
 *
 * @param {UseTableVirtualizationProps} props - Virtualization configuration and state accessors.
 * @returns {Object} { visibleRange, totalHeight } accessors.
 */
export function useTableVirtualization(props: UseTableVirtualizationProps) {
    const visibleRange = createMemo(() => {
        const overscanCount = props.overscan ?? 5;
        const scrollTop = props.scrollTop();
        const containerHeight = props.containerHeight();
        const rowHeight = props.rowHeight();

        const start = Math.max(
            0,
            Math.floor((scrollTop - props.headerHeight) / rowHeight) - overscanCount
        );
        const visibleCount = Math.ceil(containerHeight / rowHeight);
        const end = Math.min(props.dataLength(), start + visibleCount + overscanCount * 2);

        return { start, end };
    });

    const totalHeight = createMemo(() => props.dataLength() * props.rowHeight());

    return { visibleRange, totalHeight };
}
