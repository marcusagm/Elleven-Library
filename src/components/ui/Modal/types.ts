import { JSX } from 'solid-js';

/**
 * Supported size variants for the Modal component.
 */
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | 'full';

/**
 * Base properties for Modal components that share visibility state.
 */
export interface ModalBaseProperties {
    /** Whether the modal is currently visible. */
    isOpen: boolean;
    /** Callback function invoked when the modal requests to close. */
    onClose: () => void;
}

/**
 * Properties for the core Modal component.
 */
export interface ModalProperties extends ModalBaseProperties {
    /** The title displayed in the modal header. */
    title?: string;
    /** The size variant of the modal. Defaults to 'md'. */
    size?: ModalSize;
    /** Whether clicking the backdrop overlay should trigger onClose. Defaults to true. */
    closeOnOverlayClick?: boolean;
    /** Whether to display the close button in the header. Defaults to true. */
    showCloseButton?: boolean;
    /** The primary content of the modal. */
    children: JSX.Element;
    /** Optional content to be displayed in the modal footer. */
    footer?: JSX.Element;
    /** Additional CSS class names for the modal container. */
    class?: string;
}

/**
 * Properties for the ConfirmModal specialized component.
 */
export interface ConfirmModalProperties extends ModalBaseProperties {
    /** Callback function invoked when the user confirms the action. */
    onConfirm: () => void;
    /** The title of the confirmation dialog. */
    title: string;
    /** The message or description of the action to be confirmed. */
    message: string;
    /** Custom text for the confirmation button. Defaults to 'Confirm'. */
    confirmText?: string;
    /** Custom text for the cancellation button. Defaults to 'Cancel'. */
    cancelText?: string;
    /** Semantic Kind of the confirmation, affecting the confirm button style. */
    kind?: 'danger' | 'warning' | 'info';
    /** The size variant of the modal. */
    size?: ModalSize;
    /** Optional additional content to display below the message. */
    children?: JSX.Element;
}

/**
 * Properties for the PromptModal specialized component.
 */
export interface PromptModalProperties extends ModalBaseProperties {
    /** Callback function invoked when the user confirms the input. */
    onConfirm: (value: string) => void;
    /** The title displayed in the modal header. */
    title: string;
    /** A descriptive text displayed above the input field. */
    description?: string;
    /** The initial value for the input field. */
    initialValue?: string;
    /** Placeholder text for the input field. */
    placeholder?: string;
    /** Custom text for the confirmation button. Defaults to 'Confirm'. */
    confirmText?: string;
    /** Custom text for the cancellation button. Defaults to 'Cancel'. */
    cancelText?: string;
    /**
     * Optional validation function.
     * Should return an error message string if the value is invalid, or null/undefined if valid.
     */
    validate?: (value: string) => string | undefined | null;
    /** An external error message to be displayed. */
    errorMessage?: string;
    /** Whether the input is required. Defaults to false. */
    required?: boolean;
}
