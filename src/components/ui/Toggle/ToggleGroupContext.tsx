import { createContext, useContext } from 'solid-js';
import { ToggleGroupContextValue } from './types';

/**
 * Context for sharing ToggleGroup state and configuration with child items.
 */
export const ToggleGroupContext = createContext<ToggleGroupContextValue>();

/**
 * Hook to consume the ToggleGroup context.
 * Must be used within a ToggleGroup provider.
 *
 * @throws {Error} If used outside of a ToggleGroup provider.
 * @returns {ToggleGroupContextValue} The toggle group context value.
 */
export const useToggleGroup = () => {
    const context = useContext(ToggleGroupContext);
    if (!context) {
        throw new Error('ToggleGroupItem must be used within a ToggleGroup');
    }
    return context;
};
