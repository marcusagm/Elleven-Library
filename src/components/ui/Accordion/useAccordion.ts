import { createContext, useContext } from 'solid-js';
import { AccordionContextValue, AccordionItemContextValue } from './types';

/**
 * React Context object for the root Accordion component.
 * Provides access to the global expansion state and toggle functions.
 */
export const AccordionContext = createContext<AccordionContextValue>();

/**
 * Custom hook to access the context provided by the nearest <Accordion /> parent.
 *
 * @returns {AccordionContextValue} The shared state and actions for the accordion system.
 * @throws {Error} If called outside of an Accordion component tree.
 *
 * @example
 * ```ts
 * const { type, toggleItem } = useAccordion();
 * ```
 */
export const useAccordion = (): AccordionContextValue => {
    const rootContext = useContext(AccordionContext);

    if (!rootContext) {
        throw new Error(
            '[Mundam] Accordion components must be used within an <Accordion /> component.'
        );
    }

    return rootContext;
};

/**
 * React Context object for an individual AccordionItem.
 * Provides item-specific state such as expansion status, unique identifiers, and disabled state.
 */
export const AccordionItemContext = createContext<AccordionItemContextValue>();

/**
 * Custom hook to access the context provided by the nearest <AccordionItem /> parent.
 *
 * @returns {AccordionItemContextValue} The specific state for the current accordion section.
 * @throws {Error} If called outside of an AccordionItem component.
 *
 * @example
 * ```ts
 * const { isExpanded, value } = useAccordionItem();
 * ```
 */
export const useAccordionItem = (): AccordionItemContextValue => {
    const itemContext = useContext(AccordionItemContext);

    if (!itemContext) {
        throw new Error(
            '[Mundam] Accordion item sub-components must be used within an <AccordionItem /> component.'
        );
    }

    return itemContext;
};
