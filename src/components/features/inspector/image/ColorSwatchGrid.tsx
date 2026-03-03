import { Component, For, createSignal } from 'solid-js';
import { type ExtractedColorData } from './colorHarmonyUtils';

interface ColorSwatchGridProperties {
    colors: ExtractedColorData[];
}

/**
 * Grid of color swatches extracted from the image palette.
 * Clicking a swatch copies its hex value to the clipboard with visual feedback.
 */
export const ColorSwatchGrid: Component<ColorSwatchGridProperties> = properties => {
    const [copiedSwatchIndex, setCopiedSwatchIndex] = createSignal<number | null>(null);

    const handleSwatchClick = async (hexColor: string, swatchIndex: number) => {
        try {
            await navigator.clipboard.writeText(hexColor);
            setCopiedSwatchIndex(swatchIndex);
            setTimeout(() => setCopiedSwatchIndex(null), 1500);
        } catch {
            // Clipboard API may fail in some contexts; silently ignore
        }
    };

    return (
        <div class="color-swatch-grid">
            <For each={properties.colors}>
                {(color, index) => (
                    <div
                        class="color-swatch"
                        style={{ 'background-color': color.hex_color }}
                        title={`${color.hex_color} — ${(color.percentage * 100).toFixed(1)}%`}
                        aria-label={`Color swatch ${color.hex_color}`}
                        role="button"
                        tabindex={0}
                        onClick={() => handleSwatchClick(color.hex_color, index())}
                        onKeyDown={event => {
                            if (event.key === 'Enter' || event.key === ' ') {
                                event.preventDefault();
                                handleSwatchClick(color.hex_color, index());
                            }
                        }}
                    >
                        {copiedSwatchIndex() === index() && (
                            <span class="color-swatch-copied">✓</span>
                        )}
                    </div>
                )}
            </For>
        </div>
    );
};
