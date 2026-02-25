import { Accessor, JSX } from 'solid-js';
import { HueSaturationBrightness } from './utils';

/**
 * Properties for the ColorPicker component.
 */
export interface ColorPickerProps {
    /** Current color value in hexadecimal format (e.g., "#FF0000") or "transparent" */
    color: string;
    /** Callback triggered when the color changes */
    onChange: (color: string) => void;
    /** List of preset hexadecimal colors to display as quick choices */
    presets?: string[];
    /** Whether to show the hexadecimal text input field */
    showInput?: boolean;
    /** Additional CSS class names for the root element */
    class?: string;
    /** Whether to allow selecting "no color" (transparent) */
    allowNoColor?: boolean;
    /** Optional children for custom layouts using Compound Components */
    children?: JSX.Element;
}

/**
 * Internal state context for ColorPicker sub-components.
 */
export interface ColorPickerContextValue {
    /** Accessor for the current HSB state */
    hueSaturationBrightness: Accessor<HueSaturationBrightness>;
    /** Accessor for the active color string (hex or transparent) */
    activeColor: Accessor<string>;
    /** Accessor for the hexadecimal input text (can be partial or invalid) */
    activeHexadecimalInput: Accessor<string>;
    /** Method to update the hexadecimal input text */
    setHexadecimalInput: (value: string) => void;
    /** Method to update the color based on HSB changes */
    updateColorFromHueSaturationBrightness: (newValues: Partial<HueSaturationBrightness>) => void;
    /** Method to set the color directly (e.g., from presets or input) */
    setColor: (hexadecimalOrTransparent: string) => void;
    /** Whether a drag operation is currently in progress */
    isDragging: Accessor<boolean>;
    /** Method to set dragging state */
    setIsDragging: (dragging: boolean) => void;
    /** Whether "no color" option is enabled */
    allowNoColor: Accessor<boolean>;
}
