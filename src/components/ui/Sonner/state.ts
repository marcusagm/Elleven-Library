import { createSignal } from 'solid-js';
import { ToastProperties, ToastState } from './types';

/**
 * Internal global state for tracking active toasts.
 */
const [toastState, setToastState] = createSignal<ToastState>({ activeToasts: [] });

/**
 * Sequential counter for generating unique toast identifiers.
 */
let toastCounterReference = 0;

/**
 * Removes a specific toast from the global state by its identifier.
 *
 * @param toastId - The unique identifier of the toast to remove.
 */
const removeToastByIdentifier = (toastId: string): void => {
    setToastState(previousState => ({
        activeToasts: previousState.activeToasts.filter(toast => toast.identifier !== toastId)
    }));
};

/**
 * Adds a new toast to the global state with optional configurations.
 *
 * @param configuration - Initial configuration without the identifier.
 * @returns The unique identifier assigned to the new toast.
 */
const createToast = (configuration: Omit<ToastProperties, 'identifier'>): string => {
    const identifier = `toast-${++toastCounterReference}`;
    const newToast: ToastProperties = {
        identifier,
        duration: 15000,
        isDismissible: true,
        ...configuration
    };

    setToastState(previousState => ({
        activeToasts: [...previousState.activeToasts, newToast]
    }));

    return identifier;
};

/**
 * Public singleton API for triggering toast notifications from anywhere in the application.
 *
 * @example
 * toast.success("Operation completed successfully!", {
 *     description: "The file was uploaded to the cloud.",
 *     duration: 5000
 * });
 */
export const toast = {
    /**
     * Standard informational toast without a specific urgency level.
     */
    default: (
        title: string,
        options?: Partial<Omit<ToastProperties, 'identifier' | 'variant' | 'title'>>
    ) => createToast({ variant: 'default', title, ...options }),

    /**
     * Success toast used for positive feedback after an operation.
     */
    success: (
        title: string,
        options?: Partial<Omit<ToastProperties, 'identifier' | 'variant' | 'title'>>
    ) => createToast({ variant: 'success', title, ...options }),

    /**
     * Error toast used to indicate a critical failure or invalid input.
     */
    error: (
        title: string,
        options?: Partial<Omit<ToastProperties, 'identifier' | 'variant' | 'title'>>
    ) => createToast({ variant: 'error', title, ...options }),

    /**
     * Warning toast for non-blocking issues that require user attention.
     */
    warning: (
        title: string,
        options?: Partial<Omit<ToastProperties, 'identifier' | 'variant' | 'title'>>
    ) => createToast({ variant: 'warning', title, ...options }),

    /**
     * Informational toast for general updates or non-critical messages.
     */
    info: (
        title: string,
        options?: Partial<Omit<ToastProperties, 'identifier' | 'variant' | 'title'>>
    ) => createToast({ variant: 'info', title, ...options }),

    /**
     * Manually dismiss a specific active toast by its identifier.
     */
    dismiss: (identifier: string) => removeToastByIdentifier(identifier),

    /**
     * Immediately clears all active notifications from the screen.
     */
    dismissAll: () => setToastState({ activeToasts: [] })
};

/**
 * Hook or function to get the current global toast state.
 * @internal
 */
export const getActiveToastState = () => toastState();

/**
 * Internal utility to remove a toast.
 * @internal
 */
export const removeToastReference = removeToastByIdentifier;
