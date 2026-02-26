/**
 * Tooltip Component
 *
 * @module Tooltip
 * @description
 * High-quality, accessible tooltip system built with a compound component architecture.
 * Supports viewport-aware positioning, focus trapping, and click-outside dismissal.
 *
 * @example
 * ```tsx
 * import { Tooltip, TooltipRoot, TooltipTrigger, TooltipContent } from '@/components/ui';
 *
 * <TooltipRoot placement="bottom-start" offsetValue={12}>
 *   <TooltipTrigger>
 *     <Button>Open Menu</Button>
 *   </TooltipTrigger>
 *   <TooltipContent>
 *     <div role="listbox">
 *       <button onClick={() => handleAction('Option 1')}>Option 1</button>
 *       <button onClick={() => handleAction('Option 2')}>Option 2</button>
 *     </div>
 *   </TooltipContent>
 * </TooltipRoot>
 * ```
 */

import { Component, JSX } from 'solid-js';
import { Placement } from '@floating-ui/dom';
import { TooltipRoot } from './TooltipRoot';
import { TooltipTrigger } from './TooltipTrigger';
import { TooltipContent } from './TooltipContent';
import { TooltipRootProperties } from './types';

export * from './types';
export * from './TooltipRoot';
export * from './TooltipTrigger';
export * from './TooltipContent';
export * from './TooltipContext';

/**
 * Shorthand properties for the Tooltip component.
 */
export interface TooltipShorthandProperties extends TooltipRootProperties {
    /** The content to be displayed within the tooltip. */
    content: JSX.Element;
    /** @deprecated Use 'placement' instead. Mapping for legacy property. */
    position?: Placement;
}

/**
 * A shorthand component for simple tooltips.
 *
 * @param {TooltipShorthandProperties} properties - Combined properties.
 * @returns {JSX.Element} The composed Tooltip system.
 */
export const Tooltip: Component<TooltipShorthandProperties> = properties => {
    return (
        <TooltipRoot
            isVisible={properties.isVisible}
            isDefaultVisible={properties.isDefaultVisible}
            onVisibleChange={properties.onVisibleChange}
            showDelay={properties.showDelay}
            placement={properties.placement ?? properties.position}
            offsetValue={properties.offsetValue}
            isDisabled={properties.isDisabled}
        >
            <TooltipTrigger>{properties.children}</TooltipTrigger>
            <TooltipContent>{properties.content}</TooltipContent>
        </TooltipRoot>
    );
};
