import { Component } from 'solid-js';
import { useModalContext } from './ModalContext';

/**
 * Properties for the ModalOverlay component.
 */
interface ModalOverlayProperties {
    /** Whether clicking the overlay should close the modal. Defaults to true. */
    closeOnClick?: boolean;
}

/**
 * Background overlay for the Modal.
 *
 * @param componentProperties - Properties for the ModalOverlay.
 * @returns The rendered overlay element.
 */
export const ModalOverlay: Component<ModalOverlayProperties> = componentProperties => {
    const { onClose } = useModalContext();

    /**
     * Handles the click event on the overlay.
     * Prevents closing if the click happened on a child element (though overlay usually has no children).
     *
     * @param event - The mouse event.
     */
    const handleOverlayClick = (event: MouseEvent) => {
        if (event.target === event.currentTarget && (componentProperties.closeOnClick ?? true)) {
            onClose();
        }
    };

    return <div class="ui-modal-overlay" onClick={handleOverlayClick} aria-hidden="true" />;
};
