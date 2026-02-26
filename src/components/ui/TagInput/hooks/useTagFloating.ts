import { createSignal, createEffect, onCleanup, Accessor } from 'solid-js';
import { computePosition, flip, shift, offset, autoUpdate, size } from '@floating-ui/dom';

/**
 * Hook to manage the floating behavior and positioning of the suggestions dropdown.
 * Uses @floating-ui/dom for accurate, viewport-aware dropdown placement.
 *
 * @param referenceRef - Ref to the reference element (input container).
 * @param floatingRef - Ref to the suggestions dropdown element.
 * @param isVisible - Accessor indicating if the dropdown is currently visible.
 * @returns An object containing the current position and action functions.
 */
export const useTagFloating = (
    referenceRef: () => HTMLElement | undefined,
    floatingRef: () => HTMLElement | undefined,
    isVisible: Accessor<boolean>
) => {
    const [suggestionDropdownCoordinates, setSuggestionDropdownCoordinates] = createSignal({
        top: 0,
        left: 0,
        width: 0
    });

    /**
     * Recalculates the position of the suggestions dropdown portal based on current dimensions.
     */
    const updateDropdownPosition = async () => {
        const reference = referenceRef();
        const floating = floatingRef();

        if (!reference || !floating || !isVisible()) {
            return;
        }

        const { x: coordinateX, y: coordinateY } = await computePosition(reference, floating, {
            placement: 'bottom-start',
            middleware: [
                offset(4),
                flip(),
                shift({ padding: 8 }),
                size({
                    apply({ rects, elements }) {
                        // Dynamically match the dropdown width to the input container.
                        Object.assign(elements.floating.style, {
                            width: `${rects.reference.width}px`
                        });
                    }
                })
            ]
        });

        setSuggestionDropdownCoordinates({
            top: coordinateY,
            left: coordinateX,
            width: reference.offsetWidth
        });
    };

    // Automatically monitor position and size changes to sync placement.
    createEffect(() => {
        const reference = referenceRef();
        const floating = floatingRef();

        if (isVisible() && reference && floating) {
            const cleanup = autoUpdate(reference, floating, updateDropdownPosition);
            onCleanup(cleanup);
        }
    });

    return {
        suggestionDropdownCoordinates,
        updateDropdownPosition
    };
};
