/**
 * Floating UI Positioning Primitive
 *
 * Provides a standardized way to position floating elements (tooltips, popovers, menus)
 * relative to a trigger element using @floating-ui/dom for viewport-aware layout.
 */

import { createSignal, createEffect, onCleanup, Accessor } from 'solid-js';
import { computePosition, flip, shift, offset, autoUpdate, Placement } from '@floating-ui/dom';

/**
 * Common configuration for floating element behavior.
 */
export interface FloatingConfiguration {
    /** Placement of the floating element relative to the trigger. */
    placement: Accessor<Placement>;
    /** Distance in pixels between trigger and floating element. */
    offsetValue?: number | Accessor<number>;
    /** Padding to maintain from viewport edges. */
    viewportPadding?: number | Accessor<number>;
    /** Whether to automatically update position on scroll/resize. */
    isAutoUpdateEnabled?: Accessor<boolean>;
}

/**
 * Returns reactive coordinates and refs for a floating element.
 *
 * @param {FloatingConfiguration} configuration - Logic configuration.
 * @returns {Object} Accessors and setters for positioning.
 *
 * @example
 * const { setTriggerReference, setFloatingElement, coordinates } = createFloating({
 *   placement: () => 'top'
 * });
 */
export const createFloating = (configuration: FloatingConfiguration) => {
    /** Reference to the trigger (anchor) element. */
    const [triggerReference, setTriggerReference] = createSignal<HTMLElement | null>(null);
    /** Reference to the floating (content) element. */
    const [floatingElement, setFloatingElement] = createSignal<HTMLElement | null>(null);
    /** Coordinates for absolute or fixed positioning. */
    const [coordinates, setCoordinates] = createSignal({ x: 0, y: 0 });

    /**
     * Updates the position of the floating element.
     */
    const updatePosition = async () => {
        const reference = triggerReference();
        const floating = floatingElement();

        if (!reference || !floating) {
            return;
        }

        const offsetVal =
            typeof configuration.offsetValue === 'function'
                ? configuration.offsetValue()
                : (configuration.offsetValue ?? 8);
        const paddingVal =
            typeof configuration.viewportPadding === 'function'
                ? configuration.viewportPadding()
                : (configuration.viewportPadding ?? 8);

        const { x: positionX, y: positionY } = await computePosition(reference, floating, {
            placement: configuration.placement(),
            middleware: [offset(offsetVal), flip(), shift({ padding: paddingVal })]
        });

        setCoordinates({ x: positionX, y: positionY });
    };

    /**
     * Handles automatic position updates while the element is visible.
     */
    createEffect(() => {
        const reference = triggerReference();
        const floating = floatingElement();
        const isEnabled = configuration.isAutoUpdateEnabled?.() ?? true;

        if (isEnabled && reference && floating) {
            const cleanupAutoUpdate = autoUpdate(reference, floating, updatePosition);
            onCleanup(cleanupAutoUpdate);
        }
    });

    return {
        /** Signal accessor for computed coordinates. */
        coordinates,
        /** Accessor for the trigger element. */
        triggerReference,
        /** Accessor for the floating element. */
        floatingElement,
        /** Ref setter for the trigger element. */
        setTriggerReference,
        /** Ref setter for the floating element. */
        setFloatingElement,
        /** Forces a position recalculation. */
        updatePosition
    };
};
