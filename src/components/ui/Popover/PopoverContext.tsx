/**
 * Popover Context
 *
 * Provides shared state and actions for the Popover compound component parts.
 */

import { createContext, useContext } from 'solid-js';
import { PopoverContextState } from './types';

/**
 * Solid.js Context for the Popover state.
 */
export const PopoverContext = createContext<PopoverContextState>();

/**
 * Accessor hook for the Popover context.
 *
 * @returns {PopoverContextState} The current Popover context.
 */
export const usePopoverContext = () => {
    const context = useContext(PopoverContext);
    if (!context) {
        throw new Error('usePopoverContext must be used within a PopoverRoot.');
    }
    return context;
};
