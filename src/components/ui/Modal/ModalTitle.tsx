import { Component, JSX } from 'solid-js';
import { useModalContext } from './ModalContext';
import { cn } from '../../../lib/utils';

/**
 * Properties for the ModalTitle component.
 */
interface ModalTitleProperties {
    /** The title text or content. */
    children: JSX.Element;
    /** Additional CSS classes. */
    class?: string;
}

/**
 * Semantic title component for the modal.
 * Connects with the modal container via aria-labelledby.
 *
 * @param componentProperties - Properties for the ModalTitle.
 * @returns The rendered title element.
 */
export const ModalTitle: Component<ModalTitleProperties> = componentProperties => {
    const { titleIdentifier } = useModalContext();

    return (
        <h2 id={titleIdentifier} class={cn('ui-modal-title', componentProperties.class)}>
            {componentProperties.children}
        </h2>
    );
};
