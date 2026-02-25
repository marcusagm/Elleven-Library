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
 * @returns {Object} State and methods for the ColorPicker components.
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
     */
    const initializeFromColor = (colorCode: string) => {
        if (colorCode === 'transparent') {
            setHexadecimalInput('transparent');
            // We preserve HSB state so picking a color later starts from the last valid point
        } else if (validateHexadecimalColor(colorCode)) {
            const newHueSaturationBrightness =
                convertHexadecimalToHueSaturationBrightness(colorCode);
            setHueSaturationBrightness(newHueSaturationBrightness);
            setHexadecimalInput(colorCode);
        }
    };

    // Initial sync
    createEffect(() => {
        initializeFromColor(activeColor());
    });

    /**
     * Updates the color fromHue-Saturation-Brightness changes.
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
            const newHueSaturationBrightness =
                convertHexadecimalToHueSaturationBrightness(hexadecimalOrTransparent);
            setHueSaturationBrightness(newHueSaturationBrightness);
            setHexadecimalInput(hexadecimalOrTransparent);
        }
    };

    return {
        activeColor,
        activeHexadecimalInput: hexadecimalInput,
        setHexadecimalInput,
        hueSaturationBrightness,
        updateColorFromHueSaturationBrightness,
        setColor,
        isDragging,
        setIsDragging
    };
};
