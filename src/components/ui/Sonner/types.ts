import { Component } from 'solid-js';

/**
 * Supported variant types for a toast notification.
 */
export type ToastVariant = 'default' | 'success' | 'error' | 'warning' | 'info';

/**
 * Possible placements for the toaster container on the screen.
 */
export type ToasterPosition =
    | 'top-left'
    | 'top-center'
    | 'top-right'
    | 'bottom-left'
    | 'bottom-center'
    | 'bottom-right';

/**
 * Defines a clickable action button within a toast.
 */
export interface ToastActionProperties {
    /** Text displayed on the button. */
    label: string;
    /** Callback executed when the button is pressed. */
    onClick: () => void;
}

/**
 * Configuration and state for an individual toast instance.
 */
export interface ToastProperties {
    /** Unique identifier for the toast. */
    identifier: string;
    /** Visual style of the toast. */
    variant: ToastVariant;
    /** Primary message of the toast. */
    title: string;
    /** Optional secondary text for more context. */
    description?: string;
    /** Auto-dismiss delay in milliseconds. Defaults to 15000. Set to 0 to keep open. */
    duration?: number;
    /** Whether the user can manually dismiss the toast. */
    isDismissible?: boolean;
    /** Optional action button configuration. */
    action?: ToastActionProperties;
}

/**
 * State management container for active toasts.
 */
export interface ToastState {
    /** Array of currently active toast notifications. */
    activeToasts: ToastProperties[];
}

/**
 * Properties for the Toaster container component.
 */
export interface ToasterProperties {
    /** Where the toasts should appear relative to the viewport. */
    position?: ToasterPosition;
    /** Whether to always show the full stack expanded. */
    isExpandedByDefault?: boolean;
    /** Whether to use more vibrant background colors for variants. */
    useRichColors?: boolean;
}

/**
 * Properties for the internal individual ToastItem component.
 * @internal
 */
export interface ToastItemProperties {
    /** The toast data to render. */
    toast: ToastProperties;
    /** The index of the toast in the stack. */
    index: number;
    /** Total number of active toasts in the stack. */
    totalCount: number;
    /** Whether the entire stack is currently expanded. */
    isStackExpanded: boolean;
}

/**
 * Internal map for toast variant icons.
 * @internal
 */
export type ToastIconMap = Record<ToastVariant, Component<{ size?: number }>>;
