import { JSX, Accessor } from 'solid-js';

/**
 * Orientation of the slider component.
 */
export type SliderOrientation = 'horizontal' | 'vertical';

/**
 * Properties for the Slider component.
 */
export interface SliderProperties extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
    /** The current value of the slider. */
    value?: number;
    /** The default value when the component is uncontrolled. */
    defaultValue?: number;
    /** Callback fired when the value changes during interaction. */
    onValueChange?: (value: number) => void;
    /** Callback fired when the interaction is finished (pointer up or key commit). */
    onValueCommit?: (value: number) => void;
    /** The minimum allowed value. Defaults to 0. */
    min?: number;
    /** The maximum allowed value. Defaults to 100. */
    max?: number;
    /** The step interval between values. Defaults to 1. */
    step?: number;
    /** Whether the slider is disabled. */
    disabled?: boolean;
    /** The visual orientation of the slider. */
    orientation?: SliderOrientation;
    /** Whether to show a tooltip above the thumb with the current value. */
    showTooltip?: boolean;
    /** Whether to show tick marks at each step. */
    showTicks?: boolean;
    /** Function to format the value for display in tooltips and ARIA attributes. */
    formatValue?: (value: number) => string;
}

/**
 * Context state for the Slider component and its children.
 */
export interface SliderContextValue {
    /** Accessor for the current numeric value. */
    value: Accessor<number>;
    /** Accessor for the minimum value. */
    minimumValue: Accessor<number>;
    /** Accessor for the maximum value. */
    maximumValue: Accessor<number>;
    /** Accessor for the step value. */
    stepValue: Accessor<number>;
    /** Accessor for the percentage of the value within the range (0-100). */
    percentage: Accessor<number>;
    /** Whether the slider is currently being dragged. */
    isDragging: Accessor<boolean>;
    /** Update the dragging state. */
    setIsDragging: (isDragging: boolean) => void;
    /** Whether the slider is disabled. */
    isDisabled: Accessor<boolean>;
    /** The orientation of the slider. */
    orientation: Accessor<SliderOrientation>;
    /** Function to format the value. */
    formatValue: (value: number) => string;
    /** Method to update the value. */
    setValue: (value: number) => void;
    /** Method to commit the current value. */
    commitValue: (value: number) => void;
    /** Reference to the track element. */
    trackReference: { ref: HTMLDivElement | undefined };
    /** Unique ID for the slider components. */
    sliderIdentifier: Accessor<string>;
}
