import { Component, JSX } from 'solid-js';
import { cn } from '../../../lib/utils';

/**
 * Properties for the ModalBody component.
 */
interface ModalBodyProperties {
    /** The content of the modal body. */
    children: JSX.Element;
    /** Additional CSS classes. */
    class?: string;
}

/**
 * Layout component for the modal body content.
 *
 * @param componentProperties - Properties for the ModalBody.
 * @returns The rendered body element.
 */
export const ModalBody: Component<ModalBodyProperties> = componentProperties => {
    return (
        <div class={cn('ui-modal-body', componentProperties.class)}>
            {componentProperties.children}
        </div>
    );
};
