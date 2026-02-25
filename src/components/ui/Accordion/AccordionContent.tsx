import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { AccordionContentProps } from './types';
import { useAccordionItem } from './useAccordion';

/**
 * The expandable panel that holds the content of an AccordionItem.
 * Visibility and accessibility attributes are controlled automatically by the parent AccordionItem state.
 *
 * @param {AccordionContentProps} props - Configuration for the content container.
 * @returns {JSX.Element} A div element representing the collapsible region.
 *
 * @example
 * ```tsx
 * <AccordionContent>
 *   <p>This information is hidden until the section is opened.</p>
 * </AccordionContent>
 * ```
 */
export const AccordionContent: Component<AccordionContentProps> = props => {
    const [localProps, restProps] = splitProps(props, ['class', 'children']);
    const currentItemContext = useAccordionItem();

    return (
        <div
            id={currentItemContext.contentId}
            role="region"
            aria-labelledby={currentItemContext.triggerId}
            class={cn('ui-accordion-content', localProps.class)}
            data-state={currentItemContext.isExpanded() ? 'open' : 'closed'}
            hidden={!currentItemContext.isExpanded()}
            {...restProps}
        >
            <div class="ui-accordion-content-inner">{localProps.children}</div>
        </div>
    );
};
