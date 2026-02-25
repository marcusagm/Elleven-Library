import { Component, createMemo, For } from 'solid-js';
import { useSlider } from './SliderContext';

/**
 * Calculates tick values for the slider based on range and step.
 *
 * @param minValue - The minimum value of the slider.
 * @param maxValue - The maximum value of the slider.
 * @param stepValue - The step interval between ticks.
 * @returns An array of numeric values where ticks should be placed.
 */
const calculateTickValues = (minValue: number, maxValue: number, stepValue: number) => {
    const range = maxValue - minValue;
    if (range <= 0 || stepValue <= 0) return [];

    const tickCount = Math.floor(range / stepValue);
    if (tickCount > 50) return []; // Performance safeguard: Avoid too many ticks

    const tickValues: number[] = [];
    // Skip the min value as the track start represents it
    for (let index = 1; index <= tickCount; index++) {
        const value = minValue + index * stepValue;
        // Don't include if it exactly equals max to avoid overlap with track end
        if (value < maxValue) {
            tickValues.push(value);
        }
    }
    return tickValues;
};

/**
 * Component to render tick marks along the slider track.
 *
 * @returns A group of tick elements.
 */
export const SliderTicks: Component = () => {
    const slider = useSlider();

    const tickValues = createMemo(() =>
        calculateTickValues(slider.minimumValue(), slider.maximumValue(), slider.stepValue())
    );

    return (
        <For each={tickValues()}>
            {tickValue => (
                <div
                    class="ui-slider-tick"
                    style={{
                        [slider.orientation() === 'vertical' ? 'bottom' : 'left']:
                            `${((tickValue - slider.minimumValue()) / (slider.maximumValue() - slider.minimumValue())) * 100}%`
                    }}
                />
            )}
        </For>
    );
};
