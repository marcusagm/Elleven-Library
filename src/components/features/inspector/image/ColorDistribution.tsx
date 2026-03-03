import { Component, For, Show, createMemo } from 'solid-js';
import { type ExtractedColorData, hexToHsl } from './colorHarmonyUtils';

interface ColorDistributionProperties {
    colors: ExtractedColorData[];
}

/** Tolerance in degrees for grouping similar hues together. */
const HUE_GROUPING_TOLERANCE = 25;

/** Minimum saturation to consider a color chromatic when grouping. */
const MINIMUM_CHROMATIC_SATURATION_FOR_GROUPING = 0.08;

/** A grouped color family with its aggregated percentage. */
interface ColorGroup {
    representativeHex: string;
    totalPercentage: number;
    representativeHue: number;
}

/**
 * Groups extracted colors by hue proximity, merging similar colors.
 * Neutral/achromatic colors are grouped into a single "neutral" bucket.
 *
 * @param colors - All extracted colors from the palette.
 * @returns Array of color groups sorted by percentage descending.
 */
function groupColorsByHue(colors: ExtractedColorData[]): ColorGroup[] {
    if (colors.length === 0) return [];

    const groups: {
        hue: number;
        totalPercentage: number;
        dominantHex: string;
        dominantPercentage: number;
    }[] = [];

    let neutralPercentage = 0;
    let neutralDominantHex = '#808080';
    let neutralDominantPercentage = 0;

    for (const color of colors) {
        const hsl = hexToHsl(color.hex_color);

        // Achromatic colors (very low saturation) go into a neutral bucket
        if (hsl.saturation < MINIMUM_CHROMATIC_SATURATION_FOR_GROUPING) {
            neutralPercentage += color.percentage;
            if (color.percentage > neutralDominantPercentage) {
                neutralDominantPercentage = color.percentage;
                neutralDominantHex = color.hex_color;
            }
            continue;
        }

        // Find an existing group within hue tolerance
        const matchingGroup = groups.find(
            group => angularHueDifference(group.hue, hsl.hue) <= HUE_GROUPING_TOLERANCE
        );

        if (matchingGroup) {
            matchingGroup.totalPercentage += color.percentage;
            // Keep the hex of the most dominant color in the group as representative
            if (color.percentage > matchingGroup.dominantPercentage) {
                matchingGroup.dominantPercentage = color.percentage;
                matchingGroup.dominantHex = color.hex_color;
            }
        } else {
            groups.push({
                hue: hsl.hue,
                totalPercentage: color.percentage,
                dominantHex: color.hex_color,
                dominantPercentage: color.percentage
            });
        }
    }

    // Add neutral group if it has any percentage
    if (neutralPercentage > 0) {
        groups.push({
            hue: -1,
            totalPercentage: neutralPercentage,
            dominantHex: neutralDominantHex,
            dominantPercentage: neutralDominantPercentage
        });
    }

    // Sort by total percentage descending
    groups.sort((groupA, groupB) => groupB.totalPercentage - groupA.totalPercentage);

    return groups.map(group => ({
        representativeHex: group.dominantHex,
        totalPercentage: group.totalPercentage,
        representativeHue: group.hue
    }));
}

/**
 * Calculates the angular difference between two hue values on the color wheel.
 */
function angularHueDifference(hueA: number, hueB: number): number {
    const rawDifference = Math.abs(hueA - hueB);
    return rawDifference > 180 ? 360 - rawDifference : rawDifference;
}

/**
 * Horizontal stacked bar showing proportional color distribution.
 * Groups similar colors by hue proximity and shows aggregated percentages.
 */
export const ColorDistribution: Component<ColorDistributionProperties> = properties => {
    const colorGroups = createMemo(() => groupColorsByHue(properties.colors));

    const normalizedGroups = createMemo(() => {
        const groups = colorGroups();
        if (groups.length === 0) return [];

        const totalPercentage = groups.reduce((sum, group) => sum + group.totalPercentage, 0);

        if (totalPercentage <= 0) return [];

        return groups.map(group => ({
            ...group,
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
