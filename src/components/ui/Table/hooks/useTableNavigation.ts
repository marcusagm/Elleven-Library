import { Accessor } from 'solid-js';
import { useCommands } from '../../../../core/input';

/**
 * Properties for the useTableNavigation hook.
 */
interface UseTableNavigationProps<T> {
    /** Reactive accessor for the complete dataset displayed in the table */
    data: Accessor<T[]>;
    /** Reactive accessor for the index currently holding focus */
    focusedIndex: Accessor<number>;
    /** Function to update the focused index position */
    setFocusedIndex: (index: number) => void;
    /** Optional accessor for the row click handler callback */
    onRowClick?: Accessor<((item: T, multi: boolean, range: boolean) => void) | undefined>;
    /** Optional accessor for the row double-click handler callback */
    onRowDoubleClick?: Accessor<((item: T) => void) | undefined>;
    /** Optional function to scroll a specific index into view */
    scrollToIndex?: (index: number) => void;
}

/**
 * Custom hook to manage keyboard navigation and command handling for the Table.
 *
 * Integrates with the core input system (`useCommands`) to handle standard viewport
 * actions. This allows the table to respond to globally defined shortcuts like
 * moving up/down, home/end, selection toggles, and opening items without
 * direct DOM event listeners.
 *
 * @template T - The record type for the table data rows.
 * @param {UseTableNavigationProps<T>} props - State accessors and control callbacks.
 *
 * @example
 * useTableNavigation({
 *   data: items,
 *   focusedIndex,
 *   setFocusedIndex,
 *   onRowClick: () => (item) => select(item.id)
 * });
 */
export function useTableNavigation<T>(props: UseTableNavigationProps<T>) {
    /**
     * Registers standard viewport command handlers.
     * These mappings allow the table to participate in the application's unified input system.
     */
    useCommands({
        'viewport:move-down': () => {
            const currentDataLength = props.data().length;
            if (currentDataLength === 0) {
                return;
            }
            /** Move one index down, capped at the end of the collection */
            const nextIndexCandidate = Math.min(currentDataLength - 1, props.focusedIndex() + 1);
            if (nextIndexCandidate !== props.focusedIndex()) {
                props.setFocusedIndex(nextIndexCandidate);
                props.scrollToIndex?.(nextIndexCandidate);
            }
        },
        'viewport:move-up': () => {
            const currentDataLength = props.data().length;
            if (currentDataLength === 0) {
                return;
            }
            /** Move one index up, capped at the start of the collection */
            const previousIndexCandidate = Math.max(0, props.focusedIndex() - 1);
            if (previousIndexCandidate !== props.focusedIndex()) {
                props.setFocusedIndex(previousIndexCandidate);
                props.scrollToIndex?.(previousIndexCandidate);
            }
        },
        'viewport:home': () => {
            const currentDataLength = props.data().length;
            if (currentDataLength === 0) {
                return;
            }
            /** Jump directly to the first possible index */
            if (props.focusedIndex() !== 0) {
                props.setFocusedIndex(0);
                props.scrollToIndex?.(0);
            }
        },
        'viewport:end': () => {
            const currentDataLength = props.data().length;
            if (currentDataLength === 0) {
                return;
            }
            /** Jump directly to the last possible index */
            const lastPossibleIndex = currentDataLength - 1;
            if (props.focusedIndex() !== lastPossibleIndex) {
                props.setFocusedIndex(lastPossibleIndex);
                props.scrollToIndex?.(lastPossibleIndex);
            }
        },
        'viewport:toggle-select': payload => {
            const currentDataLength = props.data().length;
            const currentFocusedIndex = props.focusedIndex();
            if (
                currentDataLength === 0 ||
                currentFocusedIndex < 0 ||
                currentFocusedIndex >= currentDataLength
            ) {
                return;
            }

            const targetedItem = props.data()[currentFocusedIndex];
            /** Command payloads provide metadata about modifiers (Shift, Ctrl, etc.) */
            const activeModifiers = payload.meta.modifiers as string[] | undefined;
            const isShiftKeyPressed = activeModifiers?.includes('Shift') ?? false;

            /** Trigger row click with current state. Multi-select is inferred from modifiers. */
            props.onRowClick?.()?.(targetedItem, isShiftKeyPressed, false);
        },
        'viewport:open': () => {
            const currentDataLength = props.data().length;
            const currentFocusedIndex = props.focusedIndex();
            if (
                currentDataLength === 0 ||
                currentFocusedIndex < 0 ||
                currentFocusedIndex >= currentDataLength
            ) {
                return;
            }

            const targetedItem = props.data()[currentFocusedIndex];
            props.onRowDoubleClick?.()?.(targetedItem);
        }
    });
}
