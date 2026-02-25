import { createContext, useContext } from 'solid-js';
import { ColorPickerContextValue } from './types';

export const ColorPickerContext = createContext<ColorPickerContextValue>();

/**
 * Hook to access the ColorPicker context in sub-components.
 *
 * @returns {ColorPickerContextValue} The shared state and actions for the ColorPicker.
 * @throws {Error} If used outside of a ColorPicker provider.
 */
export const useColorPickerContext = () => {
    const context = useContext(ColorPickerContext);
    if (!context) {
        throw new Error('ColorPicker sub-components must be used within a <ColorPicker />');
    }
    return context;
};
