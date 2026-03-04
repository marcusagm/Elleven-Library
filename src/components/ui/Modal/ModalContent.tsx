import { Component, JSX, createEffect } from 'solid-js';
import { useModalContext } from './ModalContext';
import { ModalSize } from './types';
import { cn } from '../../../lib/utils';
import { createFocusTrap } from '../../../lib/primitives';
import { useShortcuts, createConditionalScope } from '../../../core/input';

/**
 * Properties for the ModalContent component.
 */
interface ModalContentProperties {
    /** The content of the modal dialog. */
    children: JSX.Element;
    /** Size variant for the modal container. */
    size?: ModalSize;
    /** Additional CSS classes for the container. */
    class?: string;
    /** ARIA role for the modal container. Defaults to 'dialog'. */
    role?: 'dialog' | 'alertdialog';
}

/**
 * The main container for the modal dialog content.
 * Handles focus trapping, keyboard shortcuts, and ARIA attributes.
 *
 * @param componentProperties - Properties for the ModalContent.
 * @returns The rendered modal container.
 */
export const ModalContent: Component<ModalContentProperties> = componentProperties => {
    const { isOpen, onClose, titleIdentifier } = useModalContext();
    let containerReference: HTMLDivElement | undefined;

    // Focus trap implementation.
    createFocusTrap(() => containerReference, isOpen);

    // Input System Integration.
    // Creates a conditional scope for the modal which is active when the modal is open.
    createConditionalScope('modal', isOpen, undefined, true);

    // Register Escape shortcut to close the modal.
    useShortcuts([
        {
            keys: 'Escape',
            name: 'Close Modal',
            scope: 'modal',
            system: true,
            enabled: isOpen,
            action: () => {
                onClose();
            }
        }
    ]);

    // Initial focus when the modal opens.
    createEffect(() => {
        if (isOpen() && containerReference) {
            containerReference.focus();
        }
    });

    /**
     * Resolves the size variant class name.
     */
    const resolvedSizeClass = () => `ui-modal-${componentProperties.size || 'md'}`;

    return (
        <div
            ref={containerReference}
            class={cn('ui-modal-container', resolvedSizeClass(), componentProperties.class)}
            role={componentProperties.role || 'dialog'}
            aria-modal="true"
            aria-labelledby={titleIdentifier}
            tabindex={-1}
        >
            {componentProperties.children}
        </div>
    );
};
