/**
 * Popover Component Types
 *
 * Defines the public interfaces and supported variants for the Popover system.
 */

import { JSX } from 'solid-js';
import { Placement } from '@floating-ui/dom';

/**
 * Alignment options for the popover relative to the trigger.
 */
export type PopoverAlignment = 'start' | 'center' | 'end';

/**
 * Side options for the popover relative to the trigger.
 */
export type PopoverSide = 'top' | 'right' | 'bottom' | 'left';

/**
 * Properties for the PopoverRoot component.
 */
export interface PopoverRootProperties {
    /** Whether the popover is currently open. */
    isOpen?: boolean;
    /** Initial open state if uncontrolled. */
    isDefaultOpen?: boolean;
    /** Callback fired when the open state changes. */
    onOpenChange?: (isOpen: boolean) => void;
    /** Whether the popover should close when clicking outside. */
    isAutoCloseEnabled?: boolean;
    /** Preferred placement of the popover. */
    placement?: Placement;
    /** Distance in pixels from the trigger. */
    offsetValue?: number;
    /** Children elements (Trigger and Content). */
    children?: JSX.Element;
}

/**
 * Properties for the PopoverContent component.
 */
export interface PopoverContentProperties {
    /** Children to render inside the popover. */
    children: JSX.Element;
    /** Additional CSS class names. */
    class?: string;
    /** Whether to trap focus inside the popover. */
    isFocusTrapEnabled?: boolean;
}

/**
 * Internal context state for the Popover compound components.
 */
export interface PopoverContextState {
    /** Reactive signal for the open state. */
    isOpen: () => boolean;
    /** Function to update the open state. */
    setIsOpen: (isOpen: boolean) => void;
    /** Unique identifier for the popover content. */
    contentId: string;
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
