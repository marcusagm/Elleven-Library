import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { AccordionRootProps, AccordionContextValue } from './types';
import { AccordionContext } from './useAccordion';
import './accordion.css';

/**
 * The Root component of the Accordion system.
 * It coordinates the expansion state of all its children and provides accessibility context.
 *
 * @param {AccordionRootProps} props - The configuration properties for the accordion.
 * @returns {JSX.Element} A reactive provider wrapping the accordion children.
 *
 * @example
 * ```tsx
 * <Accordion type="single" collapsible defaultValue={['first-item']}>
 *   <AccordionItem value="first-item">
 *     <AccordionHeader title="Section 1" />
 *     <AccordionContent>Content goes here</AccordionContent>
 *   </AccordionItem>
 * </Accordion>
 * ```
 */
export const Accordion: Component<AccordionRootProps> = props => {
    const [localProps, restProps] = splitProps(props, [
        'type',
        'value',
        'defaultValue',
        'onValueChange',
        'collapsible',
        'disabled',
        'class',
        'children'
    ]);

    /** Reactive accessor for the accordion behavior type. Defaults to 'single'. */
    const accordionType = () => localProps.type ?? 'single';

    /** Reactive accessor for whether a single expanded item can be collapsed. Defaults to true. */
    const isCollapsible = () => localProps.collapsible ?? true;

    /** Reactive accessor for the disabled state of the entire accordion. */
    const isDisabled = () => localProps.disabled ?? false;

    /**
     * Internal signal managing the list of expanded items.
     * Supports both controlled and uncontrolled modes via createControllableSignal primitive.
     */
    const { value: expandedItems, setValue: setExpandedItems } = createControllableSignal<string[]>(
        {
            value: () => localProps.value,
            defaultValue: localProps.defaultValue ?? [],
            onChange: valueList => localProps.onValueChange?.(valueList)
        }
    );

    /**
     * Toggles the expansion state of an item based on the accordion mode.
     *
     * @param {string} itemIdentifier - The unique identifier of the item to toggle.
     */
    const toggleItem = (itemIdentifier: string) => {
        if (isDisabled()) {
            return;
        }

        const currentExpandedList = expandedItems();
        const isItemCurrentlyExpanded = currentExpandedList.includes(itemIdentifier);

        if (accordionType() === 'single') {
            if (isItemCurrentlyExpanded && isCollapsible()) {
                setExpandedItems([]);
            } else if (!isItemCurrentlyExpanded) {
                setExpandedItems([itemIdentifier]);
            }
        } else {
            // Multiple items expansion mode
            if (isItemCurrentlyExpanded) {
                setExpandedItems(
                    currentExpandedList.filter(
                        (identifier: string) => identifier !== itemIdentifier
                    )
                );
            } else {
                setExpandedItems([...currentExpandedList, itemIdentifier]);
            }
        }
    };

    /** Context value provided to all descendant items and sub-components. */
    const contextValue: AccordionContextValue = {
        expandedItems,
        toggleItem,
        disabled: isDisabled,
        type: accordionType
    };

    return (
        <AccordionContext.Provider value={contextValue}>
            <div
                class={cn('ui-accordion', localProps.class)}
                data-orientation="vertical"
                data-disabled={isDisabled() ? '' : undefined}
                {...restProps}
            >
                {localProps.children}
            </div>
        </AccordionContext.Provider>
    );
};
