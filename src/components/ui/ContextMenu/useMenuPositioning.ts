/**
 * Context Menu Positioning Hook
 *
 * Provides logic for calculating the position of a context menu or its submenus.
 * Integrates with Floating UI for viewport-aware placement and collision handling.
 */

import { createSignal, createEffect, onCleanup, Accessor } from 'solid-js';
import { computePosition, flip, shift, offset, autoUpdate, Placement } from '@floating-ui/dom';

/**
 * Custom hook for managing context menu and submenu positioning.
 * Supports both virtual elements (for root context menu) and HTMLElements (for submenus).
 *
 * @param {Accessor<HTMLElement | { getBoundingClientRect(): DOMRect } | null>} reference - Trigger element or virtual point.
 * @param {Placement} placement - Desired placement relative to the reference.
 * @returns {Object} Methods to set the floating element and the computed coordinates.
 */
export const useMenuPositioning = (
    reference: Accessor<HTMLElement | { getBoundingClientRect(): DOMRect } | null>,
    placement: Placement = 'right-start'
) => {
    /** Ref for the menu content to be positioned. */
    const [floatingElement, setFloatingElement] = createSignal<HTMLElement | null>(null);
    /** Computed top/left coordinates. */
    const [coordinates, setCoordinates] = createSignal({ top: 0, left: 0 });

    /**
     * Updates the position using Floating UI computePosition.
     */
    const updatePosition = async () => {
        const referenceValue = reference();
        const floatingValue = floatingElement();

        if (!referenceValue || !floatingValue) return;

        /**
         * Determine if we are positioning a submenu.
         * Submenus should avoid flipping to bottom/top to prevent covering the parent menu.
         */
        const isSubmenu = placement === 'right-start' || placement === 'left-start';

        const { x: coordinateX, y: coordinateY } = await computePosition(
            referenceValue,
            floatingValue,
            {
                placement,
                strategy: 'fixed',
                middleware: [
                    /**
                     * For submenus, we use a negative offset to create a small overlap.
                     * This ensures the mouse never leaves a menu element when moving to a submenu,
                     * preventing premature closure without resorting to timers.
                     */
                    offset(isSubmenu ? -4 : 2),
                    flip({
                        padding: 8,
                        fallbackPlacements: isSubmenu
                            ? ['left-start']
                            : ['top-start', 'bottom-start', 'left-start']
                    }),
                    shift({ padding: 8 }) // Keep within viewport boundaries
                ]
            }
        );

        setCoordinates({ top: coordinateY, left: coordinateX });
    };

    // Automatically update position on window resize or scroll
    createEffect(() => {
        const referenceValue = reference();
        const floatingValue = floatingElement();

        if (referenceValue && floatingValue) {
            /**
             * autoUpdate handles cleanup and event listeners.
             * We use a type cast to ensure compatibility with Floating UI's expected types.
             */
            const cleanup = autoUpdate(
                referenceValue as Parameters<typeof autoUpdate>[0],
                floatingValue,
                updatePosition
            );
            onCleanup(cleanup);
        }
    });

    return {
        /** Setter for the floating menu content ref. */
        setFloatingElement,
        /** Computed coordinates accessor. */
        coordinates,
        /** Force manual position recalculation. */
        updatePosition
    };
};
