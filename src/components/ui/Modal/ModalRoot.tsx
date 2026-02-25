import { Component, JSX, Show, createEffect, onCleanup, splitProps } from 'solid-js';
import { Portal } from 'solid-js/web';
import { ModalProvider } from './ModalContext';
import { ModalBaseProperties } from './types';
import { createId } from '../../../lib/primitives/createId';

/**
 * Properties for the ModalRoot component.
 */
interface ModalRootProperties extends ModalBaseProperties {
    /** The content of the modal, typically including Overlay and Content. */
    children: JSX.Element;
}

/**
 * The root component of a Modal.
 * It provides the state context and handles the portal and body scroll locking.
 *
 * @param componentProperties - Properties for the ModalRoot.
 * @returns The rendered Portal with the Modal context.
 */
export const ModalRoot: Component<ModalRootProperties> = componentProperties => {
    const [localProperties] = splitProps(componentProperties, ['isOpen', 'onClose', 'children']);
    const titleIdentifier = createId('modal-title');

    // Handle body scroll locking when the modal is open.
    createEffect(() => {
        if (!localProperties.isOpen) {
            return;
        }

        const originalStyle = window.getComputedStyle(document.body).overflow;
        document.body.style.overflow = 'hidden';

        onCleanup(() => {
            document.body.style.overflow = originalStyle;
        });
    });

    const contextValue = {
        isOpen: () => localProperties.isOpen,
        get onClose() {
            return localProperties.onClose;
        },
        titleIdentifier
    };

    return (
        <Show when={localProperties.isOpen}>
            <Portal>
                <ModalProvider value={contextValue}>{localProperties.children}</ModalProvider>
            </Portal>
        </Show>
    );
};
