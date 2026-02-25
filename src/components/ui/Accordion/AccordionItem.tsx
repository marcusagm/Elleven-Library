import { Component, splitProps, createMemo } from 'solid-js';
import { cn } from '../../../lib/utils';
import { AccordionItemProps, AccordionItemContextValue } from './types';
import { useAccordion, AccordionItemContext } from './useAccordion';
import { createId } from '../../../lib/primitives/createId';

/**
 * Represents a single collapsible section within an Accordion.
 * It provides its unique state (expansion, disabled, IDs) to its children (Trigger and Content).
 *
 * @param {AccordionItemProps} props - Configuration for the accordion item.
 * @returns {JSX.Element} A provider wrapper for trigger and content.
 *
 * @example
 * ```tsx
 * <AccordionItem value="unique-id">
 *   <AccordionTrigger>Toggle Me</AccordionTrigger>
 *   <AccordionContent>Visible when expanded</AccordionContent>
 * </AccordionItem>
 * ```
 */
export const AccordionItem: Component<AccordionItemProps> = props => {
    const [localProps, restProps] = splitProps(props, ['value', 'disabled', 'class', 'children']);
    const accordionRootContext = useAccordion();

    /** Reactive accessor determining if this specific item is disabled (inherits from root). */
    const isItemDisabled = () => localProps.disabled || accordionRootContext.disabled();

    /** Reactive memo checking if this item's value is in the root's expanded items list. */
    const isExpanded = createMemo(() =>
        accordionRootContext.expandedItems().includes(localProps.value)
    );

    /** Unique identifiers generated for ARIA relationship linking. */
    const triggerElementId = createId();
    const contentElementId = createId();

    /** The reactive context provided to nested Trigger and Content components. */
    const itemContextValue: AccordionItemContextValue = {
        value: () => localProps.value,
        disabled: isItemDisabled,
        isExpanded,
        triggerId: triggerElementId,
        contentId: contentElementId
    };

    return (
        <AccordionItemContext.Provider value={itemContextValue}>
            <div
                class={cn(
                    'ui-accordion-item',
                    isExpanded() && 'ui-accordion-item-open',
                    isItemDisabled() && 'ui-accordion-item-disabled',
                    localProps.class
                )}
                data-state={isExpanded() ? 'open' : 'closed'}
                data-disabled={isItemDisabled() ? '' : undefined}
                {...restProps}
            >
                {localProps.children}
            </div>
        </AccordionItemContext.Provider>
    );
};
