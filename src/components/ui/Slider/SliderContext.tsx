import { createContext, useContext } from 'solid-js';
import { SliderContextValue } from './types';

/**
 * Context for sharing slider state between sub-components.
 */
export const SliderContext = createContext<SliderContextValue>();

/**
 * Hook to access the Slider context.
 *
 * @returns The slider context value.
 * @throws Error if used outside of a SliderRoot provider.
 */
export const useSlider = () => {
    const context = useContext(SliderContext);
    if (!context) {
        throw new Error('useSlider must be used within a SliderRoot component');
    }
    return context;
};
