import { Component, JSX, createMemo, createSignal, splitProps, untrack } from 'solid-js';
import { createControllableSignal } from '../../../lib/primitives';
import { createId } from '../../../lib/primitives/createId';
import { SliderContext } from './SliderContext';
import { SliderProperties } from './types';

/**
 * Properties for the SliderRoot component.
 * Extends SliderProperties to include children.
 */
interface SliderRootProperties extends SliderProperties {
    /** The child elements that will consume the slider context. */
    children: JSX.Element;
}

/**
 * The Root component for the Slider system.
 * It serves as the state orchestrator and context provider for all atomic slider parts.
 *
 * This component manages the reactive value (controlled or uncontrolled),
 * coordinate calculations state, and accessibility metadata.
 *
 * @param componentProperties - Properties for the SliderRoot, including value and range configuration.
 * @returns A context provider wrapping the slider sub-components.
 */
export const SliderRoot: Component<SliderRootProperties> = componentProperties => {
    const [localProperties] = splitProps(componentProperties, [
        'value',
        'defaultValue',
        'onValueChange',
        'onValueCommit',
        'minimumValue',
        'maximumValue',
        'stepValue',
        'isDisabled',
        'orientation',
        'formatValue',
        'id',
        'children'
    ]);

    /** Reactive identifier for the slider, used for linking labels and coordinate scopes. */
    const sliderIdentifier = createMemo(() => localProperties.id || createId('slider'));

    /** The minimum selectable value, ensuring a default of 0 if undefined. */
    const minimumValue = createMemo(() => localProperties.minimumValue ?? 0);

    /** The maximum selectable value, ensuring a default of 100 if undefined. */
    const maximumValue = createMemo(() => localProperties.maximumValue ?? 100);

    /** The step interval between values, ensuring a default of 1 if undefined. */
    const stepValue = createMemo(() => localProperties.stepValue ?? 1);

    /** The visual and interaction orientation of the slider. */
    const orientation = createMemo(() => localProperties.orientation ?? 'horizontal');

    /** Whether the entire slider system is in a disabled state. */
    const isDisabled = createMemo(() => localProperties.isDisabled ?? false);

    /** Local signal to track active pointer interaction (dragging). */
    const [isDragging, setIsDragging] = createSignal(false);

    /**
     * Managed value signal using createControllableSignal.
     * Allows the slider to work in both controlled (via value prop) and uncontrolled modes.
     */
    const { value, setValue } = createControllableSignal({
        value: () => localProperties.value,
        defaultValue: localProperties.defaultValue ?? untrack(() => minimumValue()),
        onChange: (newValue: number) => localProperties.onValueChange?.(newValue)
    });

    /**
     * Calculated percentage of the current value relative to the range.
     * Used exclusively for positioning the range indicator and the thumb handle.
     */
    const percentage = createMemo(() => {
        const range = maximumValue() - minimumValue();
        if (range === 0) return 0;
        const clampedValue = Math.min(maximumValue(), Math.max(minimumValue(), value()));
        return ((clampedValue - minimumValue()) / range) * 100;
    });

    /**
     * Formats a numeric value using the provided formatValue prop or a default string conversion.
     *
     * @param currentValue - The numeric value to format.
     * @returns The formatted string representation.
     */
    const formatValue = (currentValue: number) => {
        return localProperties.formatValue
            ? localProperties.formatValue(currentValue)
            : String(currentValue);
    };

    /**
     * Invokes the onValueCommit callback if provided.
     *
     * @param committedValue - The final value to be committed.
     */
    const commitValue = (committedValue: number) => {
        localProperties.onValueCommit?.(committedValue);
    };

    /** Mutable reference to the track element, shared via context to avoid prop drilling. */
    let trackRef: HTMLDivElement | undefined;

    /**
     * The value object provided to the SliderContext.
     * Members follow a strict naming convention and provide all necessary state/methods for sub-components.
     */
    const contextValue = {
        value,
        minimumValue,
        maximumValue,
        stepValue,
        percentage,
        isDragging,
        setIsDragging,
        isDisabled,
        orientation,
        formatValue,
        setValue,
        commitValue,
        trackReference: {
            get ref() {
                return trackRef;
            },
            set ref(element: HTMLDivElement | undefined) {
                trackRef = element;
            }
        },
        sliderIdentifier
    };

    return (
        <SliderContext.Provider value={contextValue}>
            {localProperties.children}
        </SliderContext.Provider>
    );
};
