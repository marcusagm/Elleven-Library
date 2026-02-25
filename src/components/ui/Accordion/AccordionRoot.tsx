import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { AccordionRootProps, AccordionContextValue } from './types';
import { AccordionContext } from './useAccordion';
import './accordion.css';

/**
 * Root component for the Accordion.
 * Manages the state of expanded items and provides it to sub-components via context.
 *
 * @param {AccordionRootProps} props - Properties for the accordion.
 * @returns {JSX.Element} The rendered Accordion root.
 *
 * @example
 * <Accordion type="multiple" defaultValue={['item-1']}>
 *   <AccordionItem value="item-1">...</AccordionItem>
 * </Accordion>
 */
export const Accordion: Component<AccordionRootProps> = props => {
    const [local, restProps] = splitProps(props, [
        'type',
        'value',
        'defaultValue',
        'onValueChange',
        'collapsible',
        'disabled',
        'class',
        'children'
    ]);

    const accordionType = () => local.type ?? 'single';
    const isCollapsible = () => local.collapsible ?? true;
    const isDisabled = () => local.disabled ?? false;

    const { value: expandedItems, setValue: setExpandedItems } = createControllableSignal<string[]>(
        {
            value: () => local.value,
            defaultValue: local.defaultValue ?? [],
            onChange: value => local.onValueChange?.(value)
        }
    );

    /**
     * Toggles an item's expanded state based on the accordion type.
     *
     * @param {string} itemValue - The value of the item to toggle.
     */
    const toggleItem = (itemValue: string) => {
        if (isDisabled()) {
            return;
        }

        const currentExpanded = expandedItems();
        const isItemExpanded = currentExpanded.includes(itemValue);

        if (accordionType() === 'single') {
            if (isItemExpanded && isCollapsible()) {
                setExpandedItems([]);
            } else if (!isItemExpanded) {
                setExpandedItems([itemValue]);
            }
        } else {
            // Multiple items mode
            if (isItemExpanded) {
                setExpandedItems(currentExpanded.filter((value: string) => value !== itemValue));
            } else {
                setExpandedItems([...currentExpanded, itemValue]);
            }
        }
    };

    const contextValue: AccordionContextValue = {
        expandedItems,
        toggleItem,
        disabled: isDisabled,
        type: accordionType
    };

    return (
        <AccordionContext.Provider value={contextValue}>
            <div
                class={cn('ui-accordion', local.class)}
                data-orientation="vertical"
                data-disabled={isDisabled() ? '' : undefined}
                {...restProps}
            >
                {local.children}
            </div>
        </AccordionContext.Provider>
    );
};
