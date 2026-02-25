import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderTooltip component.
 */
interface SliderTooltipProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional CSS class for custom styling of the tooltip container. */
    class?: string;
}

/**
 * The SliderTooltip component displays the current value of the slider in a floating box.
 * It is typically rendered as a child of the SliderThumb and appears when the slider
 * is hovered or focused.
 *
 * The tooltip uses the formatValue utility from the slider context to ensure consistent
 * value presentation across the UI and accessibility layers.
 *
 * @param componentProperties - Properties for the SliderTooltip.
 * @returns The rendered tooltip div element.
 */
export const SliderTooltip: Component<SliderTooltipProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, ['class']);
    const slider = useSlider();

    return (
        <div
            class={cn('ui-slider-tooltip', localProperties.class)}
            role="presentation"
            {...otherProperties}
        >
            {slider.formatValue(slider.value())}
        </div>
    );
};
