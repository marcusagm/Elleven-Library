import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';

/**
 * Properties for the SliderTooltip component.
 */
interface SliderTooltipProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional class name. */
    class?: string;
}

/**
 * A tooltip that displays the current slider value.
 * Usually rendered inside or aligned with the SliderThumb.
 *
 * @param componentProperties - Properties for the SliderTooltip.
 * @returns The rendered tooltip element.
 */
export const SliderTooltip: Component<SliderTooltipProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, ['class']);
    const slider = useSlider();

    return (
        <div class={cn('ui-slider-tooltip', localProperties.class)} {...otherProperties}>
            {slider.formatValue(slider.value())}
        </div>
    );
};
