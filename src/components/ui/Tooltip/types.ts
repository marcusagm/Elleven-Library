/**
 * Tooltip Component Types
 *
 * Defines the public interfaces and supported placements for the Tooltip system.
 */

import { JSX, Accessor } from 'solid-js';
import { Placement } from '@floating-ui/dom';

/**
 * Properties for the TooltipRoot component.
 */
export interface TooltipRootProperties {
    /** Whether the tooltip is currently visible. */
    isVisible?: boolean;
    /** Initial visibility state if uncontrolled. */
    isDefaultVisible?: boolean;
    /** Callback fired when the visibility changes. */
    onVisibleChange?: (isVisible: boolean) => void;
    /** Delay in milliseconds before showing the tooltip. */
    showDelay?: number;
    /** Preferred placement of the tooltip. */
    placement?: Placement;
    /** Distance in pixels from the trigger. */
    offsetValue?: number;
    /** Children elements (Trigger and Content). */
    children?: JSX.Element;
    /** Whether the tooltip is globally disabled. */
    isDisabled?: boolean;
}

/**
 * Properties for the TooltipContent component.
 */
export interface TooltipContentProperties {
    /** Content to display (string or JSX). */
    children: JSX.Element;
    /** Additional CSS class names. */
    class?: string;
}

/**
 * Internal context state for the Tooltip compound components.
 */
export interface TooltipContextState {
    /** Reactive signal for the visibility state. */
    isVisible: () => boolean;
    /** Function to show the tooltip. */
    show: () => void;
    /** Function to hide the tooltip. */
    hide: () => void;
    /** Unique identifier for the tooltip content. */
    contentId: string;
    /** Placement direction (for CSS arrow logic). */
    placement: Accessor<Placement>;
    /** Ref setter for the trigger element. */
    setTriggerReference: (element: HTMLElement | null) => void;
    /** Accessor for the trigger element. */
    triggerReference: () => HTMLElement | null;
    /** Ref setter for the floating content element. */
    setFloatingElement: (element: HTMLElement | null) => void;
    /** Accessor for the floating element. */
    floatingElement: () => HTMLElement | null;
    /** Computed coordinates for positioning. */
    coordinates: () => { x: number; y: number };
}
