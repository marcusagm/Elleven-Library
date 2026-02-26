/**
 * Tooltip Trigger
 *
 * Attaches pointer and keyboard focus events to the trigger element,
 * initiating or cancelling the tooltip visibility sequence.
 */

import { Component, JSX } from 'solid-js';
import { useTooltipContext } from './TooltipContext';

/**
 * Trigger wrapper for Tooltip content.
 *
 * @param {Object} triggerProperties - Properties for the trigger wrapper.
 * @returns {JSX.Element} Reactive interaction point.
 *
 * @example
 * <TooltipTrigger>
 *   <button>Hover Me</button>
 * </TooltipTrigger>
 */
export const TooltipTrigger: Component<{
    children: JSX.Element;
    class?: string;
}> = triggerProperties => {
    /** Access the shared Tooltip context. */
    const context = useTooltipContext();

    return (
        <div
            ref={context.setTriggerReference}
            onMouseEnter={context.show}
            onMouseLeave={context.hide}
            onFocus={context.show}
            onBlur={context.hide}
            aria-describedby={context.isVisible() ? context.contentId : undefined}
            class={triggerProperties.class}
            style={{ display: 'inline-flex' }}
        >
            {triggerProperties.children}
        </div>
    );
};
