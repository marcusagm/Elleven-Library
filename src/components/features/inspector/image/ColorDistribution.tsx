import { Component, For, Show, createMemo } from 'solid-js';
import {
    type ExtractedColorData,
    type ColorCluster,
    agglomerativeGrouping
} from './colorHarmonyUtils';

interface ColorDistributionProperties {
    colors: ExtractedColorData[];
    /** Pre-computed clusters. If provided, skips internal clustering. */
    clusters?: ColorCluster[];
}

/** Normalized group ready for rendering. */
interface NormalizedColorGroup {
    representativeHex: string;
    normalizedWidth: number;
    displayPercentage: number;
}

/**
 * Horizontal stacked bar showing proportional color distribution.
 * Uses agglomerative clustering in 3D cylindrical HSL space
 * to create 3–5 meaningful color families.
 *
 * Accepts optional pre-computed `clusters` to share the same grouping
 * result with the harmony badge (via `ColorPaletteSection`).
 */
export const ColorDistribution: Component<ColorDistributionProperties> = properties => {
    const colorGroups = createMemo((): ColorCluster[] => {
        if (properties.clusters) return properties.clusters;
        return agglomerativeGrouping(properties.colors);
    });

    const normalizedGroups = createMemo((): NormalizedColorGroup[] => {
        const groups = colorGroups();
        if (groups.length === 0) return [];

        const totalPercentage = groups.reduce((sum, group) => sum + group.totalPercentage, 0);
        if (totalPercentage <= 0) return [];

        return groups.map(group => ({
            representativeHex: group.representativeHex,
            normalizedWidth: (group.totalPercentage / totalPercentage) * 100,
            displayPercentage: group.totalPercentage * 100
        }));
    });

    return (
        <Show when={normalizedGroups().length > 0}>
            <div class="color-distribution">
                <div class="color-distribution-bar-container">
                    <For each={normalizedGroups()}>
                        {group => (
                            <div
                                class="color-distribution-segment"
                                style={{
                                    'background-color': group.representativeHex,
                                    width: `${Math.max(group.normalizedWidth, 0.5)}%`
                                }}
                                title={`${group.representativeHex} — ${group.displayPercentage.toFixed(1)}%`}
                            />
                        )}
                    </For>
                </div>

                <div class="color-distribution-legend">
                    <For each={normalizedGroups()}>
                        {group => (
                            <div class="color-distribution-legend-item">
                                <span
                                    class="color-distribution-legend-dot"
                                    style={{ 'background-color': group.representativeHex }}
                                />
                                <span>{group.displayPercentage.toFixed(1)}%</span>
                            </div>
                        )}
                    </For>
                </div>
            </div>
        </Show>
    );
};
