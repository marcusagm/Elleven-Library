import { Component } from 'solid-js';
import { useColorPickerContext } from './context';

/**
 * Preview component showing the currently selected color.
 * Supports a checkerboard background for transparent colors.
 */
export const ColorPreview: Component = () => {
    const { activeColor } = useColorPickerContext();

    const isTransparent = () => activeColor() === 'transparent';

    return (
        <div
            class="ui-color-picker-preview"
            style={{
                'background-color': isTransparent() ? 'transparent' : activeColor(),
                'background-image': isTransparent()
                    ? 'linear-gradient(45deg, #ccc 25%, transparent 25%), linear-gradient(-45deg, #ccc 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #ccc 75%), linear-gradient(-45deg, transparent 75%, #ccc 75%)'
                    : 'none',
                'background-size': '8px 8px',
                'background-position': '0 0, 0 4px, 4px -4px, -4px 0px'
            }}
            aria-label={`Selected color: ${activeColor()}`}
        />
    );
};
