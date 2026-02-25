import { Component, splitProps, createMemo } from 'solid-js';
import { cn } from '../../../lib/utils';
import { AccordionItemProps, AccordionItemContextValue } from './types';
import { useAccordion, AccordionItemContext } from './useAccordion';
import { createId } from '../../../lib/primitives/createId';

/**
 * Individual item within an Accordion.
 *
 * @param {AccordionItemProps} props - Properties for the accordion item.
 * @returns {JSX.Element} The rendered accordion item.
 */
export const AccordionItem: Component<AccordionItemProps> = props => {
    const [local, restProps] = splitProps(props, ['value', 'disabled', 'class', 'children']);
    const accordionContext = useAccordion();

    const isItemDisabled = () => local.disabled || accordionContext.disabled();

    const isExpanded = createMemo(() => accordionContext.expandedItems().includes(local.value));

    const triggerId = createId();
    const contentId = createId();

    const contextValue: AccordionItemContextValue = {
        value: () => local.value,
        disabled: isItemDisabled,
        isExpanded,
        triggerId,
        contentId
    };

    return (
        <AccordionItemContext.Provider value={contextValue}>
            <div
                class={cn(
                    'ui-accordion-item',
                    isExpanded() && 'ui-accordion-item-open',
                    isItemDisabled() && 'ui-accordion-item-disabled',
                    local.class
                )}
                data-state={isExpanded() ? 'open' : 'closed'}
                data-disabled={isItemDisabled() ? '' : undefined}
                {...restProps}
            >
                {local.children}
            </div>
        </AccordionItemContext.Provider>
    );
};
