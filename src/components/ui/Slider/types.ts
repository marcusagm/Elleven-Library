import { JSX, Accessor } from 'solid-js';

/**
 * Defines the visual and interaction orientation of the slider component.
 * - 'horizontal': The slider moves left/right.
 * - 'vertical': The slider moves up/down.
 */
export type SliderOrientation = 'horizontal' | 'vertical';

/**
 * Public properties for the Slider component and its atomic parts.
 * Follows strict naming conventions: no abbreviations and descriptive purposes.
 */
export interface SliderProperties extends Omit<JSX.HTMLAttributes<HTMLDivElement>, 'onChange'> {
    /** The current controlled numeric value of the slider. */
    value?: number;
    /** The initial value when the component is used in an uncontrolled manner. */
    defaultValue?: number;
    /** Callback function invoked whenever the slider value changes during interaction. */
    onValueChange?: (value: number) => void;
    /** Callback function invoked when the interaction is finalized (e.g., on pointer up or key commit). */
    onValueCommit?: (value: number) => void;
    /** The minimum selectable value for the slider. Defaults to 0 correctly. */
    minimumValue?: number;
    /** The maximum selectable value for the slider. Defaults to 100 correctly. */
    maximumValue?: number;
    /** The incremental step between selectable values. Defaults to 1. */
    stepValue?: number;
    /** Indicates if the slider is interactive. If true, interactions and focus are disabled. */
    isDisabled?: boolean;
    /** Determines if the slider is rendered horizontally or vertically. */
    orientation?: SliderOrientation;
    /** Controls whether an interactive tooltip is displayed above the handle showing the current value. */
    showTooltip?: boolean;
    /** Controls the visibility of visual tick marks at each step interval along the track. */
    showTicks?: boolean;
    /** Optional function to transform the numeric value into a formatted string for display and accessibility. */
    formatValue?: (value: number) => string;
}

/**
 * Encapsulates the internal state and methods shared across all atomic slider sub-components.
 * Provided via SliderContext to ensure strict separation of concerns and reactivity.
 */
export interface SliderContextValue {
    /** Accessor that returns the current numeric value of the slider. */
    value: Accessor<number>;
    /** Accessor that returns the minimum allowed value. */
    minimumValue: Accessor<number>;
    /** Accessor that returns the maximum allowed value. */
    maximumValue: Accessor<number>;
    /** Accessor that returns the defined step interval. */
    stepValue: Accessor<number>;
    /** Accessor that returns the calculated percentage (0-100) of the current value within the total range. */
    percentage: Accessor<number>;
    /** Accessor that returns true if the user is currently interacting with the slider via pointer. */
    isDragging: Accessor<boolean>;
    /** Function to update the dragging status. */
    setIsDragging: (isDragging: boolean) => void;
    /** Accessor that returns true if the slider as a whole is disabled. */
    isDisabled: Accessor<boolean>;
    /** Accessor that returns the current orientation (horizontal/vertical). */
    orientation: Accessor<SliderOrientation>;
    /** Utility function to format unknown numeric value into its display string version. */
    formatValue: (value: number) => string;
    /** Method to programmatically update the slider's value. */
    setValue: (value: number) => void;
    /** Method to commit a final value, usually triggering the onValueCommit callback. */
    commitValue: (value: number) => void;
    /** Contains a reactive reference to the physical track element for coordinate calculations. */
    trackReference: { ref: HTMLDivElement | undefined };
    /** Accessor returning a unique identifier for the slider instance, used for accessibility and input scopes. */
    sliderIdentifier: Accessor<string>;
}
