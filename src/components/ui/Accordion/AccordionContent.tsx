import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { AccordionContentProps } from './types';
import { useAccordionItem } from './useAccordion';

/**
 * Content component for an Accordion item expansion.
 *
 * @param {AccordionContentProps} props - Properties for the content container.
 * @returns {JSX.Element} The rendered content container.
 */
export const AccordionContent: Component<AccordionContentProps> = props => {
    const [local, restProps] = splitProps(props, ['class', 'children']);
    const itemContext = useAccordionItem();

    return (
        <div
            id={itemContext.contentId}
            role="region"
            aria-labelledby={itemContext.triggerId}
            class={cn('ui-accordion-content', local.class)}
            data-state={itemContext.isExpanded() ? 'open' : 'closed'}
            hidden={!itemContext.isExpanded()}
            {...restProps}
        >
            <div class="ui-accordion-content-inner">{local.children}</div>
        </div>
    );
};
