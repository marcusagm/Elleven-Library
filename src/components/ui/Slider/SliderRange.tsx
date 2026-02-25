import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderRange component.
 */
interface SliderRangeProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional CSS class for custom styling of the range indicator. */
    class?: string;
}

/**
 * The SliderRange component provides a visual indicator that fills the track between the
 * minimum value and the current handle position.
 *
 * It automatically updates its width or height based on the slider's current percentage
 * and orientation provided via context.
 *
 * @param componentProperties - Properties for the SliderRange.
 * @returns The rendered range div element with dynamic styles.
 */
export const SliderRange: Component<SliderRangeProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, ['class']);
    const slider = useSlider();

    return (
        <div
            class={cn('ui-slider-range', localProperties.class)}
            style={{
                /** Dynamically applies width for horizontal and height for vertical sliders. */
                [slider.orientation() === 'vertical' ? 'height' : 'width']:
                    `${slider.percentage()}%`
            }}
            {...otherProperties}
        />
    );
};
