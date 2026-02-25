import { Component, JSX } from 'solid-js';
import { cn } from '../../../lib/utils';

/**
 * Properties for the ModalFooter component.
 */
interface ModalFooterProperties {
    /** The content of the footer. */
    children: JSX.Element;
    /** Additional CSS classes. */
    class?: string;
}

/**
 * Layout component for the modal footer actions.
 *
 * @param componentProperties - Properties for the ModalFooter.
 * @returns The rendered footer element.
 */
export const ModalFooter: Component<ModalFooterProperties> = componentProperties => {
    return (
        <footer class={cn('ui-modal-footer', componentProperties.class)}>
            {componentProperties.children}
        </footer>
    );
};
