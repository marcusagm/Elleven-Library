import { Component, JSX, createMemo, createSignal, splitProps, untrack } from 'solid-js';
import { createControllableSignal } from '../../../lib/primitives';
import { createId } from '../../../lib/primitives/createId';
import { SliderContext } from './SliderContext';
import { SliderProperties } from './types';

/**
 * Properties for the SliderRoot component.
 */
interface SliderRootProperties extends SliderProperties {
    /** The content of the slider. */
    children: JSX.Element;
}

/**
 * The Root component for the Slider.
 * It manages state, accessibility, and provides context to children.
 *
 * @param componentProperties - Properties for the SliderRoot.
 * @returns The rendered provider with children.
 */
export const SliderRoot: Component<SliderRootProperties> = componentProperties => {
    const [localProperties] = splitProps(componentProperties, [
        'value',
        'defaultValue',
        'onValueChange',
        'onValueCommit',
        'min',
        'max',
        'step',
        'disabled',
        'orientation',
        'formatValue',
        'id',
        'children'
    ]);

    const sliderIdentifier = createMemo(() => localProperties.id || createId('slider'));
    const minimumValue = createMemo(() => localProperties.min ?? 0);
    const maximumValue = createMemo(() => localProperties.max ?? 100);
    const stepValue = createMemo(() => localProperties.step ?? 1);
    const orientation = createMemo(() => localProperties.orientation ?? 'horizontal');
    const isDisabled = createMemo(() => localProperties.disabled ?? false);

    const [isDragging, setIsDragging] = createSignal(false);

    const { value, setValue } = createControllableSignal({
        value: () => localProperties.value,
        defaultValue: localProperties.defaultValue ?? untrack(() => minimumValue()),
        onChange: (newValue: number) => localProperties.onValueChange?.(newValue)
    });

    const percentage = createMemo(() => {
        const range = maximumValue() - minimumValue();
        if (range === 0) return 0;
        const clampedValue = Math.min(maximumValue(), Math.max(minimumValue(), value()));
        return ((clampedValue - minimumValue()) / range) * 100;
    });

    const formatValue = (currentValue: number) => {
        return localProperties.formatValue
            ? localProperties.formatValue(currentValue)
            : String(currentValue);
    };

    const commitValue = (committedValue: number) => {
        localProperties.onValueCommit?.(committedValue);
    };

    // Shared reference for the track
    let trackRef: HTMLDivElement | undefined;

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
