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
 * A highly versatile and accessible slider component for selecting numeric values from a range.
 * This component follows the atomic design pattern and uses the SliderRoot provider to share state.
 *
 * It is fully compatible with keyboard navigation, screen readers, and touch devices.
 *
 * @param componentProperties - Properties for configuring the Slider's value, range, and appearance.
 * @returns The rendered Slider component.
 *
 * @example
 * ```tsx
 * <Slider
 *   defaultValue={50}
 *   minimumValue={0}
 *   maximumValue={100}
 *   stepValue={5}
 *   showTooltip={true}
 *   onValueChange={(newValue) => console.log('Current value:', newValue)}
 * />
 * ```
 */
export const Slider: Component<SliderProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'id',
        'value',
        'defaultValue',
        'onValueChange',
        'onValueCommit',
        'minimumValue',
        'maximumValue',
        'stepValue',
        'isDisabled',
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
            minimumValue={localProperties.minimumValue}
            maximumValue={localProperties.maximumValue}
            stepValue={localProperties.stepValue}
            isDisabled={localProperties.isDisabled}
            orientation={localProperties.orientation}
            formatValue={localProperties.formatValue}
        >
            <div
                class={cn(
                    'ui-slider',
                    `ui-slider-${localProperties.orientation || 'horizontal'}`,
                    localProperties.isDisabled && 'ui-slider-disabled',
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

                {/*
                  Hidden native range input.
                  Maintained for form submission compatibility and as an additional
                  accessibility context, although interaction is handled by SliderThumb.
                */}
                <input
                    type="range"
                    id={localProperties.id}
                    min={localProperties.minimumValue ?? 0}
                    max={localProperties.maximumValue ?? 100}
                    step={localProperties.stepValue ?? 1}
                    value={localProperties.value}
                    disabled={localProperties.isDisabled}
                    class="ui-slider-input"
                    tabindex={-1}
                    aria-hidden="true"
                    readOnly
                />
            </div>
        </SliderRoot>
    );
};
