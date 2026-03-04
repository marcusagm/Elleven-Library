/**
 * Popover Content
 *
 * Renders the floating popover container inside a Portal and manages its visibility,
 * positioning coordinates, and accessibility behaviors.
 */

import { Component, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn } from '../../../lib/utils';
import { createFocusTrap, createClickOutside } from '../../../lib/primitives';
import { usePopoverContext } from './PopoverContext';
import { PopoverContentProperties } from './types';
import './popover.css';

/**
 * Renders the floating content area for the Popover compound component.
 *
 * @param {PopoverContentProperties} contentProperties - Popover properties and children.
 * @returns {JSX.Element} The rendered portal content.
 *
 * @example
 * ```tsx
 * import { PopoverContent } from '@/components/ui';
 * <PopoverContent class="custom-popover">
 *   <p>Here is some popover content</p>
 * </PopoverContent>
 * ```
 */
export const PopoverContent: Component<PopoverContentProperties> = contentProperties => {
    /**
     * Access the shared Popover context.
     */
    const context = usePopoverContext();

    /**
     * Track open state for focus trap and positioning.
     */
    const isOpen = () => context.isOpen();

    /**
     * Local element references.
     */
    let containerReference: HTMLDivElement | undefined;

    /**
     * Handle click outside to close the popover.
     */
    createClickOutside(
        () => {
            const elements: HTMLElement[] = [];
            const trigger = context.triggerReference();
            if (trigger) {
                elements.push(trigger);
            }
            if (containerReference) {
                elements.push(containerReference);
            }
            return elements;
        },
        () => {
            if (isOpen()) {
                context.setIsOpen(false);
            }
        }
    );

    /**
     * Trap focus inside the popover when open.
     */
    createFocusTrap(() => containerReference, isOpen);

    return (
        <Show when={isOpen()}>
            <Portal>
                <div
                    ref={element => {
                        containerReference = element;
                        context.setFloatingElement(element);
                    }}
                    id={context.contentId}
                    role="dialog"
                    class={cn('ui-popover-content', contentProperties.class)}
                    style={{
                        position: 'fixed',
                        'z-index': 9998,
                        top: `${context.coordinates().x === 0 && context.coordinates().y === 0 ? -9999 : context.coordinates().y}px`,
                        left: `${context.coordinates().x === 0 && context.coordinates().y === 0 ? -9999 : context.coordinates().x}px`
                    }}
                >
                    {contentProperties.children}
                </div>
            </Portal>
        </Show>
    );
};
