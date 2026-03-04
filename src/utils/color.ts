/**
 * Utility functions for color conversion and validation.
 * Supports Hexadecimal, Hue-Saturation-Brightness (HSB/HSV), and Hue-Saturation-Lightness (HSL) formats.
 */

/**
 * Interface representing a color in Hue, Saturation, and Brightness format.
 */
export interface HueSaturationBrightness {
    /**
     * Hue value from 0 to 360
     *
     * @returns {number} The hue value.
     */
    hue: number;

    /**
     * Saturation value from 0 to 100
     *
     * @returns {number} The saturation value.
     */
    saturation: number;

    /**
     * Brightness value from 0 to 100
     *
     * @returns {number} The brightness value.
     */
    brightness: number;
}

/**
 * Interface representing a color in Hue, Saturation, and Lightness format.
 */
export interface HueSaturationLightness {
    /**
     * Hue value from 0 to 360
     *
     * @returns {number} The hue value.
     */
    hue: number;

    /**
     * Saturation value from 0 to 1
     *
     * @returns {number} The saturation value.
     */
    saturation: number;

    /**
     * Lightness value from 0 to 1
     *
     * @returns {number} The lightness value.
     */
    lightness: number;
}

/**
 * Validates if a string is a valid hexadecimal color code.
 * Supports both 3-digit (#RGB) and 6-digit (#RRGGBB) formats.
 *
 * @param {string} hexadecimal - The string to validate.
 * @returns {boolean} True if the string is a valid hexadecimal color.
 */
export function validateHexadecimalColor(hexadecimal: string): boolean {
    return /^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$/.test(hexadecimal);
}

/**
 * Converts a Hue-Saturation-Brightness color to a Hexadecimal color string.
 *
 * @param {number} hue - Hue value (0-360).
 * @param {number} saturation - Saturation value (0-100).
 * @param {number} brightness - Brightness value (0-100).
 * @returns {string} Hexadecimal color string (e.g., "#FF0000").
 */
export function convertHueSaturationBrightnessToHexadecimal(
    hue: number,
    saturation: number,
    brightness: number
): string {
    /**
     * Normalizes the saturation and brightness values to a range of 0 to 1.
     *
     * @param {number} saturation - The saturation value to normalize.
     * @param {number} brightness - The brightness value to normalize.
     * @returns {{normalizedSaturation: number, normalizedBrightness: number}} The normalized saturation and brightness values.
     */
    const normalizedSaturation = saturation / 100;

    /**
     * Normalizes the brightness value to a range of 0 to 1.
     *
     * @param {number} brightness - The brightness value to normalize.
     * @returns {number} The normalized brightness value.
     */
    const normalizedBrightness = brightness / 100;

    /**
     * Calculates the RGB components of the color.
     *
     * @param {number} index - The index of the RGB component to calculate.
     * @returns {number} The calculated RGB component value.
     */
    const calculateComponent = (index: number) => {
        const factor = (index + hue / 60) % 6;
        return (
            normalizedBrightness *
            (1 - normalizedSaturation * Math.max(0, Math.min(factor, 4 - factor, 1)))
        );
    };

    /**
     * Converts a value to its hexadecimal representation.
     *
     * @param {number} value - The value to convert.
     * @returns {string} The hexadecimal representation of the value.
     */
    const toHexadecimal = (value: number) =>
        Math.round(255 * value)
            .toString(16)
            .padStart(2, '0');

    return `#${toHexadecimal(calculateComponent(5))}${toHexadecimal(
        calculateComponent(3)
    )}${toHexadecimal(calculateComponent(1))}`;
}

/**
 * Converts a Hexadecimal color string to Hue-Saturation-Brightness format.
 *
 * @param {string} hexadecimal - Hexadecimal color string (e.g., "#FF0000" or "#F00").
 * @returns {HueSaturationBrightness} Color in HSB format.
 */
export function convertHexadecimalToHueSaturationBrightness(
    hexadecimal: string
): HueSaturationBrightness {
    let processedHexadecimal = hexadecimal.replace(/^#/, '');

    if (processedHexadecimal.length === 3) {
        processedHexadecimal = processedHexadecimal
            .split('')
            .map(character => character + character)
            .join('');
    }

    const red = parseInt(processedHexadecimal.slice(0, 2), 16) / 255;
    const green = parseInt(processedHexadecimal.slice(2, 4), 16) / 255;
    const blue = parseInt(processedHexadecimal.slice(4, 6), 16) / 255;
    const max = Math.max(red, green, blue);
    const min = Math.min(red, green, blue);
    const delta = max - min;

    let hue = 0;
    if (delta !== 0) {
        if (max === red) {
            hue = (green - blue) / delta;
        } else if (max === green) {
            hue = 2 + (blue - red) / delta;
        } else {
            hue = 4 + (red - green) / delta;
        }
        hue = 60 * (hue < 0 ? hue + 6 : hue);
    }

    const saturation = max ? (delta / max) * 100 : 0;
    const brightness = max * 100;

    return {
        hue,
        saturation,
        brightness
    };
}

/**
 * Normalizes a hexadecimal color string, adding '#' if missing and expanding shorthand format.
 *
 * @param {string} inputValue - The raw string input.
 * @returns {string | null} Normalized hex string or null if invalid.
 */
export function normalizeHexadecimalValue(inputValue: string): string | null {
    let processedValue = inputValue;

    if (!processedValue.startsWith('#') && /^[0-9A-Fa-f]{3,6}$/.test(processedValue)) {
        processedValue = '#' + processedValue;
    }

    if (validateHexadecimalColor(processedValue)) {
        if (processedValue.length === 4) {
            processedValue =
                '#' +
                processedValue[1] +
                processedValue[1] +
                processedValue[2] +
                processedValue[2] +
                processedValue[3] +
                processedValue[3];
        }
        return processedValue;
    }

    return null;
}

/**
 * Converts a hex color string to HSL values.
 *
 * @param {string} hexColor - Hex string like "#FF5733" or "FF5733".
 * @returns {HueSaturationLightness} An HSL object with hue (0-360), saturation (0-1), lightness (0-1).
 */
export function convertHexadecimalToHueSaturationLightness(
    hexColor: string
): HueSaturationLightness {
    const hexTrimmed = hexColor.replace('#', '');
    const red = parseInt(hexTrimmed.substring(0, 2), 16) / 255;
    const green = parseInt(hexTrimmed.substring(2, 4), 16) / 255;
    const blue = parseInt(hexTrimmed.substring(4, 6), 16) / 255;

    const maxChannel = Math.max(red, green, blue);
    const minChannel = Math.min(red, green, blue);
    const channelDelta = maxChannel - minChannel;

    const lightness = (maxChannel + minChannel) / 2;

    if (channelDelta === 0) {
        return { hue: 0, saturation: 0, lightness };
    }

    const saturation =
        lightness > 0.5
            ? channelDelta / (2 - maxChannel - minChannel)
            : channelDelta / (maxChannel + minChannel);

    let hue = 0;
    if (maxChannel === red) {
        hue = ((green - blue) / channelDelta + (green < blue ? 6 : 0)) * 60;
    } else if (maxChannel === green) {
        hue = ((blue - red) / channelDelta + 2) * 60;
    } else {
        hue = ((red - green) / channelDelta + 4) * 60;
    }

    return { hue, saturation, lightness };
}
