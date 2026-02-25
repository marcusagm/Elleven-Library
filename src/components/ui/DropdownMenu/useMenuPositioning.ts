import { createSignal, createEffect, onCleanup, Accessor, createMemo } from 'solid-js';
import { computePosition, flip, shift, offset, autoUpdate, Placement } from '@floating-ui/dom';
import { DropdownAlignment, DropdownSide } from './types';

/**
 * Custom hook for managing floating element positioning using Floating UI.
 * Handles viewport collisions, scrolling, and resizing.
 *
 * @param alignment - Accessor for horizontal/vertical alignment.
 * @param side - Accessor for the opening side.
 * @returns An object containing refs and the computed position state.
 */
export const useMenuPositioning = (
    alignment: Accessor<DropdownAlignment | undefined>,
    side: Accessor<DropdownSide | undefined>
) => {
    /** Root target that triggers the dropdown */
    const [triggerReference, setTriggerReference] = createSignal<HTMLElement | null>(null);
    /** Floating content to be positioned */
    const [floatingElement, setFloatingElement] = createSignal<HTMLElement | null>(null);
    /** Current computed top/left coordinates */
    const [coordinates, setCoordinates] = createSignal({ top: 0, left: 0 });

    /**
     * Resolves the placement string for Floating UI.
     * e.g., 'bottom-start', 'top-end', etc.
     */
    const placement = createMemo((): Placement => {
        const currentAlignment = alignment() || 'start';
        const currentSide = side() || 'bottom';

        // Simple case: centering
        if (currentAlignment === 'center') {
            return currentSide as Placement;
        }

        // Standard case: side-alignment (e.g., bottom-start)
        return `${currentSide}-${currentAlignment}` as Placement;
    });

    /**
     * Updates the position of the floating element relative to the trigger.
     */
    const updatePosition = async () => {
        const reference = triggerReference();
        const floating = floatingElement();

        if (!reference || !floating) return;

        const { x, y } = await computePosition(reference, floating, {
            placement: placement(),
            middleware: [
                offset(4), // Small gap between trigger and menu
                flip(), // Flip to opposite side if not enough space
                shift({ padding: 8 }) // Shift along the axis to stay in viewport
            ]
        });

        setCoordinates({ top: y, left: x });
    };

    // Use Floating UI's autoUpdate to handle scroll and resize automatically while open
    createEffect(() => {
        const reference = triggerReference();
        const floating = floatingElement();

        if (reference && floating) {
            const cleanup = autoUpdate(reference, floating, updatePosition);
            onCleanup(cleanup);
        }
    });

    return {
        /** Setter for the trigger element ref. */
        setTriggerReference,
        /** Setter for the floating content element ref. */
        setFloatingElement,
        /** Computed coordinates accessor. */
        coordinates,
        /** Forces a manual position update. */
        updatePosition
    };
};
