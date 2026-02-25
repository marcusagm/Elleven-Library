import { Component, createMemo, For } from 'solid-js';
import { useSlider } from './SliderContext';

/**
 * Calculates the appropriate numeric values where tick marks should be positioned along the track.
 * Includes a performance safeguard to prevent rendering an excessive number of elements.
 *
 * @param minimumValue - The lowest selectable value on the slider.
 * @param maximumValue - The highest selectable value on the slider.
 * @param stepValue - The incremental step determining the density of ticks.
 * @returns An array of numeric values corresponding to each intermediate tick position.
 */
const calculateTickValues = (minimumValue: number, maximumValue: number, stepValue: number) => {
    const rangeSize = maximumValue - minimumValue;
    if (rangeSize <= 0 || stepValue <= 0) return [];

    const calculatedTickCount = Math.floor(rangeSize / stepValue);

    /**
     * Performance safeguard: Avoid rendering too many tick elements which could
     * degrade DOM performance and visual clarity.
     */
    const MAXIMUM_TICK_LIMIT = 100;
    if (calculatedTickCount > MAXIMUM_TICK_LIMIT) return [];

    const tickValues: number[] = [];

    // Iteratively calculate values for each step increment.
    // We skip the boundaries (minimum/maximum) as they are visually represented by the track ends.
    for (let stepIndex = 1; stepIndex < calculatedTickCount; stepIndex++) {
        const tickPositionValue = minimumValue + stepIndex * stepValue;

        // Safety check to ensure we stay strictly within the numerical range.
        if (tickPositionValue < maximumValue) {
            tickValues.push(tickPositionValue);
        }
    }

    return tickValues;
};

/**
 * The SliderTicks component renders visual markers at each step interval along the slider track.
 * These markers help users perceive the granularity and selectable positions of the slider.
 *
 * @returns A reactive collection of tick DIV elements.
 */
export const SliderTicks: Component = () => {
    const slider = useSlider();

    /**
     * Memoized calculation of tick values to ensure
     * the DOM only updates when range or step configuration changes.
     */
    const tickValues = createMemo(() =>
        calculateTickValues(slider.minimumValue(), slider.maximumValue(), slider.stepValue())
    );

    return (
        <For each={tickValues()}>
            {tickValue => (
                <div
                    class="ui-slider-tick"
                    style={{
                        /** Positions the tick correctly along the track based on its percentage within the range. */
                        [slider.orientation() === 'vertical' ? 'bottom' : 'left']:
                            `${((tickValue - slider.minimumValue()) / (slider.maximumValue() - slider.minimumValue())) * 100}%`
                    }}
                />
            )}
        </For>
    );
};
