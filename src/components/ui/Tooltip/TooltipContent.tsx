/**
 * Tooltip Content
 *
 * Renders the floating tooltip container inside a Portal and manage its visibility,
 * positioning coordinates, and placement-driven visual arrows.
 */

import { Component, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn } from '../../../lib/utils';
import { useTooltipContext } from './TooltipContext';
import { TooltipContentProperties } from './types';
import './tooltip.css';

/**
 * Floating content area for the Tooltip compound component.
 *
 * @param {TooltipContentProperties} contentProperties - Tooltip properties and children.
 * @returns {JSX.Element} The rendered portal content.
 */
export const TooltipContent: Component<TooltipContentProperties> = contentProperties => {
    /** Access the shared Tooltip context. */
    const context = useTooltipContext();

    /** Resolve placement side for CSS arrow class. */
    const placementSide = () => {
        const placement = context.placement();
        return placement.split('-')[0];
    };

    return (
        <Show when={context.isVisible()}>
            <Portal>
                <div
                    ref={context.setFloatingElement}
                    id={context.contentId}
                    role="tooltip"
                    class={cn(
                        'ui-tooltip',
                        `ui-tooltip-${placementSide()}`,
                        contentProperties.class
                    )}
                    style={{
                        position: 'fixed',
                        'z-index': 10000,
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
