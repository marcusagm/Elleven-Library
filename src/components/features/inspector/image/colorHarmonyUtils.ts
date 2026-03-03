/**
 * Color harmony analysis utilities for the inspector's color palette section.
 *
 * Converts hex colors to HSL, analyzes hue relationships between dominant colors,
 * and classifies the palette into standard color harmony types.
 */

/** Supported color harmony classifications. */
export type HarmonyType =
    | 'monochromatic'
    | 'complementary'
    | 'analogous'
    | 'triadic'
    | 'split_complementary'
    | 'tetradic'
    | 'neutral'
    | 'not_identified';

/** Display metadata for each harmony type. */
interface HarmonyDisplayInfo {
    label: string;
    description: string;
}

/** Map of harmony types to their human-readable labels and descriptions. */
export const HARMONY_DISPLAY_MAP: Record<HarmonyType, HarmonyDisplayInfo> = {
    monochromatic: {
        label: 'Monochromatic',
        description: 'Variations of a single hue with different lightness and saturation'
    },
    complementary: {
        label: 'Complementary',
        description: 'Colors opposite on the color wheel (~180° apart)'
    },
    analogous: {
        label: 'Analogous',
        description: 'Colors adjacent on the color wheel (within ~30° of each other)'
    },
    triadic: {
        label: 'Triadic',
        description: 'Three colors evenly spaced (~120° apart) on the color wheel'
    },
    split_complementary: {
        label: 'Split Complementary',
        description: 'A base color plus two colors adjacent to its complement'
    },
    tetradic: {
        label: 'Tetradic',
        description: 'Four colors forming two complementary pairs (~90° apart)'
    },
    neutral: {
        label: 'Neutral',
        description: 'Colors with very low saturation (grays, whites, blacks)'
    },
    not_identified: {
        label: 'Not Identified',
        description: 'The color palette does not match a standard harmony pattern'
    }
};

/** Individual extracted color as returned by the backend. */
export interface ExtractedColorData {
    id: number;
    asset_id: number;
    hex_color: string;
    lab_lightness: number;
    lab_green_red: number;
    lab_blue_yellow: number;
    percentage: number;
    rank: number;
}

/** HSL representation of a color (hue in degrees, saturation and lightness as 0-1). */
interface HslColor {
    hue: number;
    saturation: number;
    lightness: number;
}

/** Tolerance in degrees for hue comparison. */
const HUE_TOLERANCE = 30;

/** Minimum saturation to consider a color chromatic (not neutral). */
const MINIMUM_CHROMATIC_SATURATION = 0.08;

/** Minimum percentage for a color to be considered in harmony analysis. */
const MINIMUM_HARMONY_PERCENTAGE = 0.01;

/**
 * Converts a hex color string to HSL values.
 *
 * @param hexColor - Hex string like "#FF5733" or "FF5733".
 * @returns An HSL object with hue (0-360), saturation (0-1), lightness (0-1).
 */
export function hexToHsl(hexColor: string): HslColor {
    const hexTrimmed = hexColor.replace('#', '');
    const red = parseInt(hexTrimmed.substring(0, 2), 16) / 255;
    const green = parseInt(hexTrimmed.substring(2, 4), 16) / 255;
    const blue = parseInt(hexTrimmed.substring(4, 6), 16) / 255;

    const maxChannel = Math.max(red, green, blue);
    const minChannel = Math.min(red, green, blue);
    const channelDelta = maxChannel - minChannel;

    const lightness = (maxChannel + minChannel) / 2;

    if (channelDelta === 0) {
        return { hue: 0, saturation: 0, lightness };
    }

    const saturation =
        lightness > 0.5
            ? channelDelta / (2 - maxChannel - minChannel)
            : channelDelta / (maxChannel + minChannel);

    let hue = 0;
    if (maxChannel === red) {
        hue = ((green - blue) / channelDelta + (green < blue ? 6 : 0)) * 60;
    } else if (maxChannel === green) {
        hue = ((blue - red) / channelDelta + 2) * 60;
    } else {
        hue = ((red - green) / channelDelta + 4) * 60;
    }

    return { hue, saturation, lightness };
}

/**
 * Calculates the angular difference between two hue values on the color wheel.
 *
 * @param hueA - First hue value (0-360).
 * @param hueB - Second hue value (0-360).
 * @returns The shortest angular distance (0-180).
 */
function angularHueDifference(hueA: number, hueB: number): number {
    const rawDifference = Math.abs(hueA - hueB);
    return rawDifference > 180 ? 360 - rawDifference : rawDifference;
}

/**
 * Classifies 2 hue clusters into a harmony type.
 */
function classifyTwoClusters(hueClusters: number[]): HarmonyType | null {
    const hueDelta = angularHueDifference(hueClusters[0], hueClusters[1]);
    if (hueDelta >= 150 && hueDelta <= 210) return 'complementary';
    if (hueDelta <= 60) return 'analogous';
    return null;
}

/**
 * Classifies 3 hue clusters into a harmony type.
 */
function classifyThreeClusters(hueClusters: number[]): HarmonyType | null {
    const sortedDeltas = [
        angularHueDifference(hueClusters[0], hueClusters[1]),
        angularHueDifference(hueClusters[1], hueClusters[2]),
        angularHueDifference(hueClusters[0], hueClusters[2])
    ].sort((deltaA, deltaB) => deltaA - deltaB);

    if (sortedDeltas.every(delta => delta >= 90 && delta <= 150)) return 'triadic';
    if (sortedDeltas[0] <= 60 && sortedDeltas[2] >= 140) return 'split_complementary';
    if (sortedDeltas.every(delta => delta <= 60)) return 'analogous';
    return null;
}

/**
 * Classifies 4+ hue clusters into a harmony type.
 */
function classifyFourPlusClusters(hueClusters: number[]): HarmonyType | null {
    const sortedDeltas: number[] = [];
    for (let index = 0; index < hueClusters.length; index++) {
        for (let innerIndex = index + 1; innerIndex < hueClusters.length; innerIndex++) {
            sortedDeltas.push(angularHueDifference(hueClusters[index], hueClusters[innerIndex]));
        }
    }
    sortedDeltas.sort((deltaA, deltaB) => deltaA - deltaB);

    const nearRightAngleCount = sortedDeltas.filter(
        delta => (delta >= 70 && delta <= 110) || (delta >= 160 && delta <= 200)
    ).length;
    if (nearRightAngleCount >= sortedDeltas.length * 0.6) return 'tetradic';
    return null;
}

/**
 * Classifies hue clusters into a harmony type based on count.
 */
function classifyHueClusters(hueClusters: number[]): HarmonyType {
    if (hueClusters.length === 2) return classifyTwoClusters(hueClusters) ?? 'not_identified';
    if (hueClusters.length === 3) return classifyThreeClusters(hueClusters) ?? 'not_identified';
    if (hueClusters.length >= 4) return classifyFourPlusClusters(hueClusters) ?? 'not_identified';
    return 'not_identified';
}

/**
 * Detects the color harmony type from a palette of extracted colors.
 *
 * Analyzes the top dominant colors and classifies the hue relationships.
 * If no standard harmony is found, returns 'not_identified'.
 *
 * @param colors - Array of extracted color data from the backend.
 * @returns The detected harmony type.
 */
export function detectColorHarmony(colors: ExtractedColorData[]): HarmonyType {
    if (colors.length === 0) return 'not_identified';

    // Filter to colors with at least 1% presence to avoid noise clusters
    const significantColors = colors.filter(
        color => color.percentage >= MINIMUM_HARMONY_PERCENTAGE
    );
    if (significantColors.length === 0) return 'not_identified';

    const hslColors = significantColors.map(color => ({
        hsl: hexToHsl(color.hex_color),
        percentage: color.percentage
    }));

    const chromaticColors = hslColors.filter(
        entry => entry.hsl.saturation >= MINIMUM_CHROMATIC_SATURATION
    );

    if (chromaticColors.length === 0) return 'neutral';
    if (chromaticColors.length === 1) return 'monochromatic';

    const hueValues = chromaticColors.map(entry => entry.hsl.hue);
    const isMonochromatic = hueValues.every(
        hue => angularHueDifference(hue, hueValues[0]) <= HUE_TOLERANCE
    );
    if (isMonochromatic) return 'monochromatic';

    return classifyHueClusters(clusterHueValues(hueValues));
}

/**
 * Clusters nearby hue values together (within HUE_TOLERANCE degrees).
 *
 * @param hueValues - Array of hue values in degrees.
 * @returns Array of representative hue values for each cluster.
 */
function clusterHueValues(hueValues: number[]): number[] {
    const sortedHues = [...hueValues].sort((hueA, hueB) => hueA - hueB);
    const clusters: number[][] = [];

    for (const hue of sortedHues) {
        const matchingCluster = clusters.find(
            cluster => angularHueDifference(cluster[0], hue) <= HUE_TOLERANCE
        );
        if (matchingCluster) {
            matchingCluster.push(hue);
        } else {
            clusters.push([hue]);
        }
    }

    return clusters.map(cluster => cluster.reduce((sum, hue) => sum + hue, 0) / cluster.length);
}
