import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderRange component.
 */
interface SliderRangeProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional class name. */
    class?: string;
}

/**
 * The visual range indicator of the slider, representing the selected value.
 *
 * @param componentProperties - Properties for the SliderRange.
 * @returns The rendered range element.
 */
export const SliderRange: Component<SliderRangeProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, ['class']);
    const slider = useSlider();

    return (
        <div
            class={cn('ui-slider-range', localProperties.class)}
            style={{
                [slider.orientation() === 'vertical' ? 'height' : 'width']:
                    `${slider.percentage()}%`
            }}
            {...otherProperties}
        />
    );
};
