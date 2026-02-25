import { Component, splitProps, Show } from 'solid-js';
import { cn } from '../../../lib/utils';
import { SliderRoot } from './SliderRoot';
import { SliderTrack } from './SliderTrack';
import { SliderRange } from './SliderRange';
import { SliderThumb } from './SliderThumb';
import { SliderTicks } from './SliderTicks';
import { SliderProperties } from './types';
import './slider.css';

/**
 * A versatile slider component for selecting numeric values from a range.
 * This is the simplified, backward-compatible version of the slider.
 *
 * @param componentProperties - Properties for the Slider.
 * @returns The rendered Slider component.
 *
 * @example
 * <Slider defaultValue={50} min={0} max={100} onValueChange={console.log} />
 */
export const Slider: Component<SliderProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'id',
        'value',
        'defaultValue',
        'onValueChange',
        'onValueCommit',
        'min',
        'max',
        'step',
        'disabled',
        'orientation',
        'showTooltip',
        'showTicks',
        'formatValue'
    ]);

    return (
        <SliderRoot
            id={localProperties.id}
            value={localProperties.value}
            defaultValue={localProperties.defaultValue}
            onValueChange={localProperties.onValueChange}
            onValueCommit={localProperties.onValueCommit}
            min={localProperties.min}
            max={localProperties.max}
            step={localProperties.step}
            disabled={localProperties.disabled}
            orientation={localProperties.orientation}
            formatValue={localProperties.formatValue}
        >
            <div
                class={cn(
                    'ui-slider',
                    `ui-slider-${localProperties.orientation || 'horizontal'}`,
                    localProperties.disabled && 'ui-slider-disabled',
                    localProperties.class
                )}
                {...otherProperties}
            >
                <SliderTrack>
                    <SliderRange />
                    <Show when={localProperties.showTicks !== false}>
                        <SliderTicks />
                    </Show>
                    <SliderThumb showTooltip={localProperties.showTooltip} />
                </SliderTrack>

                {/* Hidden input for form submission & accessibility context */}
                <input
                    type="range"
                    id={localProperties.id}
                    min={localProperties.min ?? 0}
                    max={localProperties.max ?? 100}
                    step={localProperties.step ?? 1}
                    value={localProperties.value}
                    disabled={localProperties.disabled}
                    class="ui-slider-input"
                    tabindex={-1}
                    aria-hidden="true"
                    readOnly
                />
            </div>
        </SliderRoot>
    );
};
