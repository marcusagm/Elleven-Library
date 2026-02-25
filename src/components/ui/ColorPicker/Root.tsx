import { Component, mergeProps, splitProps } from 'solid-js';
import { ColorPickerProps, ColorPickerContextValue } from './types';
import { useColorPicker } from './useColorPicker';
import { cn } from '../../../lib/utils';
import './color-picker.css';

import { ColorPickerContext } from './context';
import { ColorArea } from './ColorArea';
import { ColorSlider } from './ColorSlider';
import { ColorPresets } from './ColorPresets';
import { ColorPreview } from './ColorPreview';
import { ColorHexInput } from './ColorHexInput';

/**
 * Root component for the ColorPicker system.
 * Uses a Compound Component pattern to allow flexible layouts.
 *
 * @param {ColorPickerProps} properties - Component properties.
 * @returns {import('solid-js').JSX.Element} The rendered component.
 *
 * @example
 * <ColorPicker color={color()} onChange={setColor}>
 *   <ColorPicker.Area />
 *   <ColorPicker.Slider />
 *   <div class="row">
 *     <ColorPicker.Preview />
 *     <ColorPicker.HexInput />
 *   </div>
 *   <ColorPicker.Presets />
 * </ColorPicker>
 */
export const ColorPicker: Component<ColorPickerProps> & {
    Area: typeof ColorArea;
    Slider: typeof ColorSlider;
    Presets: typeof ColorPresets;
    Preview: typeof ColorPreview;
    HexInput: typeof ColorHexInput;
} = properties => {
    const merged = mergeProps({ allowNoColor: false, showInput: true }, properties);
    const [local, others] = splitProps(merged, [
        'children',
        'class',
        'allowNoColor',
        'showInput',
        'color',
        'onChange',
        'presets'
    ]);

    const {
        activeColor,
        activeHexadecimalInput,
        setHexadecimalInput,
        hueSaturationBrightness,
        updateColorFromHueSaturationBrightness,
        setColor,
        isDragging,
        setIsDragging
    } = useColorPicker(merged);

    const contextValue: ColorPickerContextValue = {
        hueSaturationBrightness,
        activeColor,
        activeHexadecimalInput,
        setHexadecimalInput,
        updateColorFromHueSaturationBrightness,
        setColor,
        isDragging,
        setIsDragging,
        allowNoColor: () => merged.allowNoColor || false
    };

    return (
        <ColorPickerContext.Provider value={contextValue}>
            <div
                class={cn('ui-color-picker', local.class)}
                onClick={event => event.stopPropagation()}
                {...others}
            >
                {local.children || (
                    <>
                        <ColorArea />
                        <ColorSlider />
                        {local.showInput && (
                            <div class="ui-color-picker-controls">
                                <ColorPreview />
                                <ColorHexInput />
                            </div>
                        )}
                        <ColorPresets presets={local.presets} />
                    </>
                )}
            </div>
        </ColorPickerContext.Provider>
    );
};

// Attach sub-components for the Compound Component pattern
ColorPicker.Area = ColorArea;
ColorPicker.Slider = ColorSlider;
ColorPicker.Presets = ColorPresets;
ColorPicker.Preview = ColorPreview;
ColorPicker.HexInput = ColorHexInput;
