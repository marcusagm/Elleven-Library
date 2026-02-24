import { Accessor } from 'solid-js';
import { useCommands } from '../../../../core/input';

interface UseTableNavigationProps<T extends Record<string, unknown>> {
    data: Accessor<T[]>;
    focusedIndex: Accessor<number>;
    setFocusedIndex: (index: number) => void;
    onRowClick?: Accessor<((item: T, multi: boolean, range: boolean) => void) | undefined>;
    onRowDoubleClick?: Accessor<((item: T) => void) | undefined>;
    scrollToIndex?: (index: number) => void;
}

/**
 * Custom hook to manage keyboard navigation and command handling for the Table.
 * Integrates with the core input system (`useCommands`) to handle viewport actions
 * like moving up/down, home/end, selection toggle, and opening items.
 *
 * @template T - The record type for the table data.
 * @param {UseTableNavigationProps<T>} props - State accessors and callbacks.
 */
export function useTableNavigation<T extends Record<string, unknown>>(
    props: UseTableNavigationProps<T>
) {
    // We register the handlers for the viewport commands defined in the input system
    useCommands({
        'viewport:move-down': () => {
            const dataLen = props.data().length;
            if (dataLen === 0) return;
            const next = Math.min(dataLen - 1, props.focusedIndex() + 1);
            if (next !== props.focusedIndex()) {
                props.setFocusedIndex(next);
                props.scrollToIndex?.(next);
            }
        },
        'viewport:move-up': () => {
            const dataLen = props.data().length;
            if (dataLen === 0) return;
            const next = Math.max(0, props.focusedIndex() - 1);
            if (next !== props.focusedIndex()) {
                props.setFocusedIndex(next);
                props.scrollToIndex?.(next);
            }
        },
        'viewport:home': () => {
            const dataLen = props.data().length;
            if (dataLen === 0) return;
            if (props.focusedIndex() !== 0) {
                props.setFocusedIndex(0);
                props.scrollToIndex?.(0);
            }
        },
        'viewport:end': () => {
            const dataLen = props.data().length;
            if (dataLen === 0) return;
            const last = dataLen - 1;
            if (props.focusedIndex() !== last) {
                props.setFocusedIndex(last);
                props.scrollToIndex?.(last);
            }
        },
        'viewport:toggle-select': payload => {
            const dataLen = props.data().length;
            const currentIndex = props.focusedIndex();
            if (dataLen === 0 || currentIndex < 0 || currentIndex >= dataLen) return;

            const item = props.data()[currentIndex];
            // Access modifiers from payload.meta (set by KeyboardToken)
            const modifiers = payload.meta.modifiers as string[] | undefined;
            const isShift = modifiers?.includes('Shift') ?? false;
            props.onRowClick?.()?.(item, isShift, false);
        },
        'viewport:open': () => {
            const dataLen = props.data().length;
            const currentIndex = props.focusedIndex();
            if (dataLen === 0 || currentIndex < 0 || currentIndex >= dataLen) return;

            const item = props.data()[currentIndex];
            props.onRowDoubleClick?.()?.(item);
        }
    });
}
