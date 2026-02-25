import { createSignal, createEffect } from 'solid-js';
import {
    HueSaturationBrightness,
    convertHexadecimalToHueSaturationBrightness,
    convertHueSaturationBrightnessToHexadecimal,
    validateHexadecimalColor
} from './utils';
import { ColorPickerProps } from './types';
import { createControllableSignal } from '../../../lib/primitives';

/**
 * Hook to manage the internal state and business logic of a ColorPicker.
 * Handles conversion between hexadecimal and HSB, dragging interactions, and keyboard navigation.
 *
 * @param {ColorPickerProps} properties - The properties passed to the ColorPicker component.
 * @returns {Object} Accessors and methods for managing the color picker state.
 */
export const useColorPicker = (properties: ColorPickerProps) => {
    // Core state: hexadecimal string (or "transparent")
    const { value: activeColor, setValue: setActiveColor } = createControllableSignal({
        value: () => properties.color,
        onChange: (newColor: string) => properties.onChange?.(newColor),
        defaultValue: '#000000'
    });

    // Reactive state for the HSB values used by visual controls
    const [hueSaturationBrightness, setHueSaturationBrightness] =
        createSignal<HueSaturationBrightness>({
            hue: 0,
            saturation: 100,
            brightness: 100
        });

    // Track if any part of the picker is being dragged
    const [isDragging, setIsDragging] = createSignal(false);

    // Internal state for the hexadecimal input field text to allow partial typing
    const [hexadecimalInput, setHexadecimalInput] = createSignal('');

    /**
     * Initializes the HSB state based on the current active color.
     *
     * @param {string} hexadecimalColorCode - The hexadecimal color code to sync from.
     */
    const initializeStateFromColor = (hexadecimalColorCode: string) => {
        if (hexadecimalColorCode === 'transparent') {
            setHexadecimalInput('transparent');
            // We preserve HSB state so picking a color later starts from the last valid point
        } else if (validateHexadecimalColor(hexadecimalColorCode)) {
            const updatedHueSaturationBrightness =
                convertHexadecimalToHueSaturationBrightness(hexadecimalColorCode);
            setHueSaturationBrightness(updatedHueSaturationBrightness);
            setHexadecimalInput(hexadecimalColorCode);
        }
    };

    // Initial sync
    createEffect(() => {
        initializeStateFromColor(activeColor());
    });

    /**
     * Updates the color from Hue-Saturation-Brightness changes.
     *
     * @param {Partial<HueSaturationBrightness>} newValues - The partial HSB values to update.
     */
    const updateColorFromHueSaturationBrightness = (
        newValues: Partial<HueSaturationBrightness>
    ) => {
        const updatedHueSaturationBrightness = { ...hueSaturationBrightness(), ...newValues };
        setHueSaturationBrightness(updatedHueSaturationBrightness);

        const hexadecimal = convertHueSaturationBrightnessToHexadecimal(
            updatedHueSaturationBrightness.hue,
            updatedHueSaturationBrightness.saturation,
            updatedHueSaturationBrightness.brightness
        );

        setHexadecimalInput(hexadecimal);
        setActiveColor(hexadecimal);
    };

    /**
     * Sets the color directly, often from a preset or text input.
     *
     * @param {string} hexadecimalOrTransparent - The color code or "transparent".
     */
    const setColor = (hexadecimalOrTransparent: string) => {
        if (hexadecimalOrTransparent.toLowerCase() === 'transparent') {
            if (properties.allowNoColor) {
                setActiveColor('transparent');
                setHexadecimalInput('transparent');
            }
            return;
        }

        if (validateHexadecimalColor(hexadecimalOrTransparent)) {
            setActiveColor(hexadecimalOrTransparent);
            const updatedHueSaturationBrightness =
                convertHexadecimalToHueSaturationBrightness(hexadecimalOrTransparent);
            setHueSaturationBrightness(updatedHueSaturationBrightness);
            setHexadecimalInput(hexadecimalOrTransparent);
        }
    };

    return {
        /** Accessor for the active hexadecimal color string */
        activeColor,
        /** Accessor for the active text input value */
        activeHexadecimalInput: hexadecimalInput,
        /** Method to set the raw text input value */
        setHexadecimalInput,
        /** Accessor for the derived HSB state */
        hueSaturationBrightness,
        /** Method to update the color from HSB adjustments */
        updateColorFromHueSaturationBrightness,
        /** Method to set a predefined color value */
        setColor,
        /** Accessor for the drag interaction status */
        isDragging,
        /** Method to update the drag status */
        setIsDragging
    };
};
