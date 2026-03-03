import { Component, createResource, createMemo, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { type AssetItem } from '../../../../types';
import { AccordionItem, AccordionHeader, AccordionContent } from '../../../ui';
import { Palette, Loader2 } from 'lucide-solid';
import {
    type ExtractedColorData,
    agglomerativeGrouping,
    detectColorHarmony
} from './colorHarmonyUtils';
import { ColorHarmonyBadge } from './ColorHarmonyBadge';
import { ColorDistribution } from './ColorDistribution';
import { ColorSwatchGrid } from './ColorSwatchGrid';
import './color-palette.css';

interface ColorPaletteSectionProperties {
    item: AssetItem;
}

/**
 * Inspector accordion section that displays the full color palette analysis.
 * Contains three subsections: harmony classification, distribution bars, and swatch grid.
 *
 * Computes agglomerative clustering once and shares the result between
 * the distribution bar and harmony badge for consistent analysis.
 */
export const ColorPaletteSection: Component<ColorPaletteSectionProperties> = properties => {
    const [colors] = createResource(
        () => properties.item.id,
        async (assetId: number) => {
            try {
                return await invoke<ExtractedColorData[]>('get_asset_colors', {
                    assetId
                });
            } catch {
                return [];
            }
        }
    );

    /** Shared agglomerative clusters — computed once, used by both distribution and harmony. */
    const colorClusters = createMemo(() => {
        const colorData = colors();
        if (!colorData || colorData.length === 0) return [];
        return agglomerativeGrouping(colorData);
    });

    const harmonyType = createMemo(() => {
        const clusters = colorClusters();
        if (clusters.length === 0) return 'not_identified' as const;
        return detectColorHarmony(clusters);
    });

    return (
        <AccordionItem value="color-palette">
            <AccordionHeader title="Color Palette" icon={<Palette size={14} />} />
            <AccordionContent>
                <Show
                    when={!colors.loading}
                    fallback={
                        <div class="inspector-loading-spinner">
                            <Loader2 class="animate-spin" size={20} />
                        </div>
                    }
                >
                    <Show
                        when={(colors() || []).length > 0}
                        fallback={<div class="inspector-no-data">No color data extracted yet.</div>}
                    >
                        <div class="color-palette-section">
                            {/* Harmony Classification */}
                            <div class="color-palette-section-row">
                                <span class="color-palette-section-row-label">Harmony</span>
                                <ColorHarmonyBadge harmonyType={harmonyType()} />
                            </div>

                            {/* Distribution Bars (receives shared clusters) */}
                            <div class="color-palette-section-row">
                                <span class="color-palette-section-row-label">Distribution</span>
                                <ColorDistribution colors={colors()!} clusters={colorClusters()} />
                            </div>

                            {/* Swatch Grid (raw colors, no clustering) */}
                            <div class="color-palette-section-row">
                                <span class="color-palette-section-row-label">Palette</span>
                                <ColorSwatchGrid colors={colors()!} />
                            </div>
                        </div>
                    </Show>
                </Show>
            </AccordionContent>
        </AccordionItem>
    );
};
