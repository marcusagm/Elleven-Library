/**
 * Popover Component
 *
 * @module Popover
 * @description
 * High-quality, accessible popover system built with a compound component architecture.
 * Supports viewport-aware positioning, focus trapping, and click-outside dismissal.
 *
 * @example
 * <PopoverRoot placement="bottom-start" offsetValue={12}>
 *   <PopoverTrigger>
 *     <Button>Open Menu</Button>
 *   </PopoverTrigger>
 *   <PopoverContent>
 *     <div role="listbox">
 *       <button onClick={() => handleAction('Option 1')}>Option 1</button>
 *       <button onClick={() => handleAction('Option 2')}>Option 2</button>
 *     </div>
 *   </PopoverContent>
 * </PopoverRoot>
 */

export * from './types';
export * from './PopoverRoot';
export * from './PopoverTrigger';
export * from './PopoverContent';
export * from './PopoverContext';

import { Component, JSX } from 'solid-js';
import { Placement } from '@floating-ui/dom';
import { PopoverRoot } from './PopoverRoot';
import { PopoverTrigger } from './PopoverTrigger';
import { PopoverContent } from './PopoverContent';
import { PopoverRootProperties } from './types';

/**
 * Shorthand properties for the Popover component.
 */
export interface PopoverShorthandProperties extends PopoverRootProperties {
    /** The trigger element that toggles the popover. */
    trigger?: JSX.Element;
    /** @deprecated Use placement. Maps to alignment part of placement. */
    align?: 'start' | 'center' | 'end';
    /** @deprecated Use placement. Maps to side part of placement. */
    side?: 'top' | 'right' | 'bottom' | 'left';
    /** Callback fired when the popover closes. Mapped from onOpenChange. */
    onClose?: () => void;
    /** Additional CSS class names for the content container. */
    contentClass?: string;
    /** Extra class for the root container (if applicable, but our Root doesn't render DOM). */
    class?: string;
}

/**
 * A shorthand component for simple popovers.
 * Provides backward compatibility with the legacy monolithic Popover component.
 *
 * @param {PopoverShorthandProperties} properties - Combined properties.
 * @returns {JSX.Element} The composed Popover system.
 */
export const Popover: Component<PopoverShorthandProperties> = properties => {
    /** Support onClose legacy prop. */
    const handleOpenChange = (isOpen: boolean) => {
        properties.onOpenChange?.(isOpen);
        if (!isOpen) {
            properties.onClose?.();
        }
    };

    /** Infer placement from side and align if not provided. */
    const inferredPlacement = () => {
        if (properties.placement) return properties.placement;
        const side = properties.side ?? 'bottom';
        const align = properties.align ?? 'start';
        return align === 'center' ? side : `${side}-${align}`;
    };

    return (
        <PopoverRoot
            isOpen={properties.isOpen}
            isDefaultOpen={properties.isDefaultOpen}
            onOpenChange={handleOpenChange}
            placement={inferredPlacement() as Placement}
            offsetValue={properties.offsetValue}
            isAutoCloseEnabled={properties.isAutoCloseEnabled}
        >
            <PopoverTrigger>{properties.trigger}</PopoverTrigger>
            <PopoverContent class={properties.contentClass ?? properties.class}>
                {properties.children}
            </PopoverContent>
        </PopoverRoot>
    );
};
