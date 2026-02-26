/**
 * Popover Trigger
 *
 * Attaches opening and closing events to the target element and links it
 * via ARIA attributes to the popover content.
 */

import { Component, JSX } from 'solid-js';
import { usePopoverContext } from './PopoverContext';

/**
 * Trigger component for the Popover systems.
 *
 * @param {Object} triggerProperties - Properties for the trigger wrapper.
 * @returns {JSX.Element} Reactive interaction point.
 *
 * @example
 * <PopoverTrigger>
 *   <button>Open Popover</button>
 * </PopoverTrigger>
 */
export const PopoverTrigger: Component<{
    children: JSX.Element;
    class?: string;
}> = triggerProperties => {
    /** Access the shared Popover context. */
    const context = usePopoverContext();

    /** Handle toggle click. */
    const handleToggleOpen = (event: MouseEvent) => {
        event.stopPropagation();
        context.setIsOpen(!context.isOpen());
    };

    return (
        <div
            ref={context.setTriggerReference}
            onClick={handleToggleOpen}
            aria-haspopup="dialog"
            aria-expanded={context.isOpen()}
            aria-controls={context.isOpen() ? context.contentId : undefined}
            class={triggerProperties.class}
            style={{ display: 'inline-block' }}
        >
            {triggerProperties.children}
        </div>
    );
};
