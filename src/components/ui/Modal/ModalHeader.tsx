import { Component, JSX } from 'solid-js';
import { cn } from '../../../lib/utils';

/**
 * Properties for the ModalHeader component.
 */
interface ModalHeaderProperties {
    /** The content of the header. */
    children: JSX.Element;
    /** Additional CSS classes. */
    class?: string;
}

/**
 * Layout component for the modal header.
 *
 * @param componentProperties - Properties for the ModalHeader.
 * @returns The rendered header element.
 */
export const ModalHeader: Component<ModalHeaderProperties> = componentProperties => {
    return (
        <header class={cn('ui-modal-header', componentProperties.class)}>
            {componentProperties.children}
        </header>
    );
};
