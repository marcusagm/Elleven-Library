import { Component, For } from 'solid-js';
import { useColorPickerContext } from './context';
import { cn } from '../../../lib/utils';

const DEFAULT_PRESET_COLORS = [
    '#ef4444',
    '#f97316',
    '#f59e0b',
    '#eab308',
    '#84cc16',
    '#22c55e',
    '#10b981',
    '#14b8a6',
    '#06b6d4',
    '#0ea5e9',
    '#3b82f6',
    '#6366f1',
    '#8b5cf6',
    '#a855f7',
    '#d946ef',
    '#ec4899',
    '#f43f5e',
    '#ffffff',
    '#94a3b8',
    '#64748b',
    '#475569',
    '#1e293b',
    '#000000'
];

/**
 * Presets component for the ColorPicker.
 * Displays a grid of color swatches and an optional "transparent" option.
 *
 * @param {Object} properties - Component properties.
 * @param {string[]} [properties.presets] - Optional list of custom hexadecimal color strings.
 * @returns {import('solid-js').JSX.Element} The rendered presets grid.
 */
export const ColorPresets: Component<{ presets?: string[] }> = properties => {
    const { activeColor, setColor, allowNoColor } = useColorPickerContext();

    const presetColorList = () => properties.presets ?? DEFAULT_PRESET_COLORS;

    return (
        <div class="ui-color-picker-presets" role="listbox" aria-label="Color presets">
            {allowNoColor() && (
                <button
                    type="button"
                    class={cn(
                        'ui-color-picker-preset',
                        'ui-color-picker-preset-transparent',
                        activeColor() === 'transparent' && 'ui-color-picker-preset-selected'
                    )}
                    onClick={() => setColor('transparent')}
                    title="No Color"
                    role="option"
                    aria-selected={activeColor() === 'transparent'}
                    aria-label="Transparent"
                />
            )}
            <For each={presetColorList()}>
                {hexadecimalColor => (
                    <button
                        type="button"
                        class={cn(
                            'ui-color-picker-preset',
                            activeColor().toLowerCase() === hexadecimalColor.toLowerCase() &&
                                'ui-color-picker-preset-selected'
                        )}
                        style={{ 'background-color': hexadecimalColor }}
                        onClick={() => setColor(hexadecimalColor)}
                        title={hexadecimalColor}
                        role="option"
                        aria-selected={
                            activeColor().toLowerCase() === hexadecimalColor.toLowerCase()
                        }
                        aria-label={hexadecimalColor}
                    />
                )}
            </For>
        </div>
    );
};
