import { Accessor, JSX } from 'solid-js';
import { HueSaturationBrightness } from './utils';

/**
 * Properties for the ColorPicker component.
 */
export interface ColorPickerProps {
    /**
     * Current color value in hexadecimal format (e.g., "#FF0000") or "transparent"
     *
     * @type {string}
     */
    color: string;

    /**
     * Callback triggered when the color changes
     *
     * @type {(color: string) => void}
     */
    onChange: (color: string) => void;

    /**
     * List of preset hexadecimal colors to display as quick choices
     *
     * @type {string[]}
     */
    presets?: string[];

    /**
     * Whether to show the hexadecimal text input field
     *
     * @type {boolean}
     */
    showInput?: boolean;

    /**
     * Additional CSS class names for the root element
     *
     * @type {string}
     */
    class?: string;

    /**
     * Whether to allow selecting "no color" (transparent)
     *
     * @type {boolean}
     */
    allowNoColor?: boolean;

    /**
     * Optional children for custom layouts using Compound Components
     *
     * @type {JSX.Element}
     */
    children?: JSX.Element;
}

/**
 * Internal state context for ColorPicker sub-components.
 */
export interface ColorPickerContextValue {
    /**
     * Accessor for the current HSB state
     *
     * @type {Accessor<HueSaturationBrightness>}
     */
    hueSaturationBrightness: Accessor<HueSaturationBrightness>;

    /**
     * Accessor for the active color string (hex or transparent)
     *
     * @type {Accessor<string>}
     */
    activeColor: Accessor<string>;

    /**
     * Accessor for the hexadecimal input text (can be partial or invalid)
     *
     * @type {Accessor<string>}
     */
    activeHexadecimalInput: Accessor<string>;

    /**
     * Method to update the hexadecimal input text
     *
     * @type {(value: string) => void}
     */
    setHexadecimalInput: (value: string) => void;

    /**
     * Method to update the color based on HSB changes
     *
     * @type {(newValues: Partial<HueSaturationBrightness>) => void}
     */
    updateColorFromHueSaturationBrightness: (newValues: Partial<HueSaturationBrightness>) => void;

    /**
     * Method to set the color directly (e.g., from presets or input)
     *
     * @type {(hexadecimalOrTransparent: string) => void}
     */
    setColor: (hexadecimalOrTransparent: string) => void;

    /**
     * Whether a drag operation is currently in progress
     *
     * @type {Accessor<boolean>}
     */
    isDragging: Accessor<boolean>;

    /**
     * Method to set dragging state
     *
     * @type {(dragging: boolean) => void}
     */
    setIsDragging: (dragging: boolean) => void;

    /**
     * Whether "no color" option is enabled
     *
     * @type {Accessor<boolean>}
     */
    allowNoColor: Accessor<boolean>;
}
