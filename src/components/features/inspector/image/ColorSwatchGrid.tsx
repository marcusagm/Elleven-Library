import { Component, For, createSignal, createMemo } from 'solid-js';
import { type ExtractedColorData } from './colorHarmonyUtils';
import { ContextMenu, type ContextMenuItem } from '../../../ui/ContextMenu';
import { generateColorFormats } from '../../../../utils/color';

interface ColorSwatchGridProperties {
    colors: ExtractedColorData[];
}

/**
 * Grid of color swatches extracted from the image palette.
 * Clicking a swatch copies its hex value to the clipboard with visual feedback.
 * Right-clicking opens a context menu to copy the color in various formats.
 */
export const ColorSwatchGrid: Component<ColorSwatchGridProperties> = properties => {
    const [copiedSwatchIndex, setCopiedSwatchIndex] = createSignal<number | null>(null);
    const [contextMenuState, setContextMenuState] = createSignal<{
        isOpen: boolean;
        coordinateX: number;
        coordinateY: number;
        colorIndex: number | null;
        colorHex: string | null;
    }>({
        isOpen: false,
        coordinateX: 0,
        coordinateY: 0,
        colorIndex: null,
        colorHex: null
    });

    const handleSwatchClick = async (copiedValue: string, swatchIndex: number) => {
        try {
            await navigator.clipboard.writeText(copiedValue);
            setCopiedSwatchIndex(swatchIndex);
            setTimeout(() => setCopiedSwatchIndex(null), 1500);
            setContextMenuState(previous => ({ ...previous, isOpen: false }));
        } catch {
            // Clipboard API may fail in some contexts; silently ignore
        }
    };

    const handleContextMenu = (event: MouseEvent, hexColor: string, index: number) => {
        event.preventDefault();
        setContextMenuState({
            isOpen: true,
            coordinateX: event.clientX,
            coordinateY: event.clientY,
            colorIndex: index,
            colorHex: hexColor
        });
    };

    const contextMenuItems = createMemo<ContextMenuItem[]>(() => {
        const state = contextMenuState();
        if (!state.isOpen || state.colorHex === null || state.colorIndex === null) {
            return [];
        }

        const formats = generateColorFormats(state.colorHex);
        const index = state.colorIndex;

        return formats.map(format => ({
            type: 'item',
            label: `Copy as ${format.label} (${format.value})`,
            action: () => handleSwatchClick(format.value, index)
        }));
    });

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
                        onContextMenu={event => handleContextMenu(event, color.hex_color, index())}
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

            <ContextMenu
                coordinateX={contextMenuState().coordinateX}
                coordinateY={contextMenuState().coordinateY}
                isOpen={contextMenuState().isOpen}
                items={contextMenuItems()}
                onClose={() => setContextMenuState(previous => ({ ...previous, isOpen: false }))}
            />
        </div>
    );
};
