import { Component } from 'solid-js';
import { X } from 'lucide-solid';
import { useModalContext } from './ModalContext';
import { cn } from '../../../lib/utils';

/**
 * Properties for the ModalCloseButton component.
 */
interface ModalCloseButtonProperties {
    /** Additional CSS classes. */
    class?: string;
    /** Accessibility label for the close button. Defaults to 'Close modal'. */
    ariaLabel?: string;
}

/**
 * Close button component for the modal header.
 *
 * @param componentProperties - Properties for the ModalCloseButton.
 * @returns The rendered close button.
 */
export const ModalCloseButton: Component<ModalCloseButtonProperties> = componentProperties => {
    const { onClose } = useModalContext();

    return (
        <button
            type="button"
            class={cn('ui-modal-close', componentProperties.class)}
            onClick={onClose}
            aria-label={componentProperties.ariaLabel || 'Close modal'}
        >
            <X size={18} />
        </button>
    );
};
