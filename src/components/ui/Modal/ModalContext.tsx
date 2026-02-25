import { createContext, useContext, Accessor } from 'solid-js';

/**
 * Context properties for the Modal component tree.
 */
export interface ModalContextValue {
    /** Accessor for the current open state of the modal. */
    isOpen: Accessor<boolean>;
    /** Callback function to request closing the modal. */
    onClose: () => void;
    /** Unique identifier for the modal title to be used with aria-labelledby. */
    titleIdentifier: string;
}

const ModalContext = createContext<ModalContextValue>();

/**
 * Hook to access the Modal context.
 * Must be used within a ModalRoot component.
 *
 * @returns The modal context value.
 * @throws Error if used outside of a ModalProvider.
 */
export function useModalContext(): ModalContextValue {
    const context = useContext(ModalContext);
    if (!context) {
        throw new Error('useModalContext must be used within a ModalRoot');
    }
    return context;
}

export const ModalProvider = ModalContext.Provider;
