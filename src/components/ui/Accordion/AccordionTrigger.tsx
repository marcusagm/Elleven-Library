import { Component, splitProps, JSX, Show } from 'solid-js';
import { ChevronRight } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import { AccordionTriggerProps } from './types';
import { useAccordion, useAccordionItem } from './useAccordion';

/**
 * Trigger component for an Accordion item.
 * Must be used inside an AccordionItem.
 *
 * @param {AccordionTriggerProps} props - Properties for the trigger.
 * @returns {JSX.Element} The rendered trigger button.
 */
export const AccordionTrigger: Component<AccordionTriggerProps> = props => {
    const [local, restProps] = splitProps(props, ['class', 'children']);
    const accordionContext = useAccordion();
    const itemContext = useAccordionItem();

    const handleToggle = () => {
        if (!itemContext.disabled()) {
            accordionContext.toggleItem(itemContext.value());
        }
    };

    return (
        <button
            type="button"
            id={itemContext.triggerId}
            class={cn('ui-accordion-trigger', local.class)}
            aria-expanded={itemContext.isExpanded()}
            aria-controls={itemContext.contentId}
            aria-disabled={itemContext.disabled()}
            disabled={itemContext.disabled()}
            onClick={handleToggle}
            {...restProps}
        >
            {local.children}
        </button>
    );
};

/**
 * Convenience header component that includes the standard chevron, title, and optional icon.
 * Follows the Mundam design pattern while being part of the compound component structure.
 */
export const AccordionHeader: Component<{
    title: string | JSX.Element;
    icon?: JSX.Element;
    class?: string;
}> = props => {
    return (
        <AccordionTrigger class={props.class}>
            <span class="ui-accordion-trigger-content">
                <AccordionChevron />
                <span class="ui-accordion-title">{props.title}</span>
            </span>
            <Show when={props.icon}>
                <span class="ui-accordion-icon" aria-hidden="true">
                    {props.icon}
                </span>
            </Show>
        </AccordionTrigger>
    );
};

/**
 * Default chevron icon for the Accordion.
 * Rotates automatically when the item is expanded.
 */
export const AccordionChevron: Component<{ class?: string; size?: number }> = props => {
    return <ChevronRight size={props.size ?? 16} class={cn('ui-accordion-chevron', props.class)} />;
};
