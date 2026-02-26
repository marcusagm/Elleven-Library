/**
 * Slider component
 *
 * @module Slider
 * @description
 * The Slider component is a specialized component for displaying sliders.
 *
 * @example
 * <Slider
 *   min={0}
 *   max={100}
 *   step={1}
 *   value={50}
 *   onChange={(value) => console.log(value)}
 *   class="custom-class"
 *   wrapperClass="custom-wrapper-class"
 *   label="custom-label"
 *   error={false}
 *   errorMessage="custom-error-message"
 *   size="md"
 * />
 */
export * from './types';
export * from './SliderContext';
export * from './SliderRoot';
export * from './SliderTrack';
export * from './SliderRange';
export * from './SliderThumb';
export * from './SliderTicks';
export * from './SliderTooltip';
export * from './Slider';
