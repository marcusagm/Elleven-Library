/**
 * Tooltip Context
 *
 * Provides shared state, actions, and positioning identifiers for the Tooltip system.
 */

import { createContext, useContext } from 'solid-js';
import { TooltipContextState } from './types';

/**
 * Solid.js Context for the Tooltip state.
 */
export const TooltipContext = createContext<TooltipContextState>();

/**
 * Accessor hook for the Tooltip context.
 *
 * @returns {TooltipContextState} The current Tooltip context.
 */
export const useTooltipContext = () => {
    const context = useContext(TooltipContext);
    if (!context) {
        throw new Error('useTooltipContext must be used within a TooltipRoot.');
    }
    return context;
};
