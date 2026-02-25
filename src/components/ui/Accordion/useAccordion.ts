import { createContext, useContext } from 'solid-js';
import { AccordionContextValue, AccordionItemContextValue } from './types';

/**
 * Context for the root Accordion component.
 */
export const AccordionContext = createContext<AccordionContextValue>();

/**
 * Accesses the Accordion root context.
 * @throws {Error} If used outside an Accordion component.
 */
export const useAccordion = () => {
    const context = useContext(AccordionContext);
    if (!context) {
        throw new Error('[Mundam] Accordion components must be used within an <Accordion />');
    }
    return context;
};

/**
 * Context for an individual Accordion item.
 */
export const AccordionItemContext = createContext<AccordionItemContextValue>();

/**
 * Accesses the current AccordionItem context.
 */
export const useAccordionItem = () => {
    const context = useContext(AccordionItemContext);
    if (!context) {
        throw new Error(
            '[Mundam] Accordion sub-components must be used within an <AccordionItem />'
        );
    }
    return context;
};
