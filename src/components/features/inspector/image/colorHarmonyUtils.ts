/**
 * Color harmony classification utilities.
 *
 * Analyzes agglomerative clusters to classify palettes into standard
 * color harmony types (complementary, triadic, analogous, etc.).
 *
 * Re-exports clustering types so consumers can import everything
 * from this single module.
 */
import { type ColorCluster, type CartesianColorPoint } from './colorClusteringUtils';

export type { ExtractedColorData, CartesianColorPoint, ColorCluster } from './colorClusteringUtils';
export { agglomerativeGrouping } from './colorClusteringUtils';

/**
 * Supported color harmony classifications.
 *
 * @enum {string}
 * @property {string} monochromatic - Variations of a single hue with different lightness and saturation
 * @property {string} complementary - Colors opposite on the color wheel (~180° apart)
 * @property {string} analogous - Colors adjacent on the color wheel (within ~30° of each other)
 * @property {string} triadic - Three colors evenly spaced (~120° apart) on the color wheel
 * @property {string} split_complementary - A base color plus two colors adjacent to its complement
 * @property {string} tetradic - Four colors forming two complementary pairs (rectangular spacing)
 * @property {string} square - Four colors evenly spaced at 90° intervals on the color wheel
 * @property {string} dyadic - Two colors separated by ~60°, creating subtle contrast
 * @property {string} accented_analogous - An analogous group with one complementary accent color for contrast
 * @property {string} achromatic - Strictly grayscale — only black, white, and pure grays
 * @property {string} neutral - Colors with very low saturation (beiges, off-whites, muted tones)
 * @property {string} polychromatic - Many distinct colors spread across the color wheel
 * @property {string} not_identified - The color palette does not match a standard harmony pattern
 */
export type HarmonyType =
    | 'monochromatic'
    | 'complementary'
    | 'analogous'
    | 'triadic'
    | 'split_complementary'
    | 'tetradic'
    | 'square'
    | 'dyadic'
    | 'accented_analogous'
    | 'achromatic'
    | 'neutral'
    | 'polychromatic'
    | 'not_identified';

/**
 * Display metadata for each harmony type.
 *
 * @interface HarmonyDisplayInfo
 * @property {string} label - The human-readable label for the harmony type
 * @property {string} description - The description of the harmony type
 */
interface HarmonyDisplayInfo {
    label: string;
    description: string;
}

/**
 * Map of harmony types to their human-readable labels and descriptions.
 *
 * @type {Record<HarmonyType, HarmonyDisplayInfo>}
 */
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
        description: 'Four colors forming two complementary pairs (rectangular spacing)'
    },
    square: {
        label: 'Square',
        description: 'Four colors evenly spaced at 90° intervals on the color wheel'
    },
    dyadic: {
        label: 'Dyadic',
        description: 'Two colors separated by ~60°, creating subtle contrast'
    },
    accented_analogous: {
        label: 'Accented Analogous',
        description: 'An analogous group with one complementary accent color for contrast'
    },
    achromatic: {
        label: 'Achromatic',
        description: 'Strictly grayscale — only black, white, and pure grays'
    },
    neutral: {
        label: 'Neutral',
        description: 'Colors with very low saturation (beiges, off-whites, muted tones)'
    },
    polychromatic: {
        label: 'Polychromatic',
        description: 'Many distinct colors spread across the color wheel'
    },
    not_identified: {
        label: 'Not Identified',
        description: 'The color palette does not match a standard harmony pattern'
    }
};

/**
 * Minimum saturation to consider a cluster chromatic (not neutral).
 *
 * @type {number}
 */
const MINIMUM_CHROMATIC_SATURATION = 0.08;

/**
 * Saturation threshold below which a cluster is strictly achromatic (pure gray).
 *
 * @type {number}
 */
const ACHROMATIC_SATURATION_THRESHOLD = 0.03;

/**
 * Calculates the angular difference between two hue values on the color wheel.
 *
 * @param {number} hueA - The first hue value (0-360).
 * @param {number} hueB - The second hue value (0-360).
 * @returns {number} The angular difference between the two hue values.
 */
function angularHueDifference(hueA: number, hueB: number): number {
    const rawDifference = Math.abs(hueA - hueB);
    return rawDifference > 180 ? 360 - rawDifference : rawDifference;
}

/**
 * Extracts the representative hue from a cluster's centroid in 3D space.
 * Reconstructs the hue angle from the cartesian x,y coordinates via atan2.
 *
 * @param {CartesianColorPoint} centroid - The centroid of the color cluster.
 * @returns {number} The hue angle in degrees (0-360).
 */
function centroidToHue(centroid: CartesianColorPoint): number {
    const hueRadians = Math.atan2(centroid.coordinateY, centroid.coordinateX);
    const hueDegrees = (hueRadians * 180) / Math.PI;
    return hueDegrees < 0 ? hueDegrees + 360 : hueDegrees;
}

/**
 * Calculates the saturation magnitude from a cluster's centroid.
 * Distance from the z-axis in cylindrical coordinates.
 *
 * @param {CartesianColorPoint} centroid - The centroid of the color cluster.
 * @returns {number} The saturation value (0-1).
 */
function centroidToSaturation(centroid: CartesianColorPoint): number {
    const xSquared = centroid.coordinateX * centroid.coordinateX;
    const ySquared = centroid.coordinateY * centroid.coordinateY;
    return Math.sqrt(xSquared + ySquared);
}

/**
 * Classifies 2 hue clusters into a harmony type.
 *
 * @param {number[]} hueValues - The hue values of the two clusters.
 * @returns {HarmonyType | null} The harmony type.
 */
function classifyTwoClusters(hueValues: number[]): HarmonyType | null {
    const hueDelta = angularHueDifference(hueValues[0], hueValues[1]);
    if (hueDelta >= 150 && hueDelta <= 210) return 'complementary';
    if (hueDelta >= 45 && hueDelta <= 80) return 'dyadic';
    if (hueDelta <= 45) return 'analogous';
    return null;
}

/**
 * Checks if 3 colors form an accented analogous harmony.
 * Two colors must be analogous (≤45°) and the third roughly complementary
 * (~180°) to the analogous pair's midpoint.
 *
 * @param {number[]} hueValues - The hue values of the three clusters.
 * @returns {boolean} True if the colors form an accented analogous harmony.
 */
function isAccentedAnalogous(hueValues: number[]): boolean {
    for (let baseIndex = 0; baseIndex < 3; baseIndex++) {
        const otherIndices = [0, 1, 2].filter(index => index !== baseIndex);
        const pairDelta = angularHueDifference(
            hueValues[otherIndices[0]],
            hueValues[otherIndices[1]]
        );

        if (pairDelta > 45) continue;

        const pairMidpoint = (hueValues[otherIndices[0]] + hueValues[otherIndices[1]]) / 2;
        const accentDelta = angularHueDifference(hueValues[baseIndex], pairMidpoint);

        if (accentDelta >= 140 && accentDelta <= 210) return true;
    }
    return false;
}

/**
 * Classifies 3 hue clusters into a harmony type.
 *
 * @param {number[]} hueValues - The hue values of the three clusters.
 * @returns {HarmonyType | null} The harmony type.
 */
function classifyThreeClusters(hueValues: number[]): HarmonyType | null {
    const sortedDeltas = [
        angularHueDifference(hueValues[0], hueValues[1]),
        angularHueDifference(hueValues[1], hueValues[2]),
        angularHueDifference(hueValues[0], hueValues[2])
    ].sort((deltaA, deltaB) => deltaA - deltaB);

    if (sortedDeltas.every(delta => delta >= 90 && delta <= 150)) return 'triadic';
    if (isAccentedAnalogous(hueValues)) return 'accented_analogous';
    if (sortedDeltas[0] <= 60 && sortedDeltas[2] >= 140) return 'split_complementary';
    if (sortedDeltas.every(delta => delta <= 60)) return 'analogous';
    return null;
}

/**
 * Checks if 4 colors form a square harmony.
 * In a perfect square: 4 pairwise deltas ~90° and 2 deltas ~180°.
 *
 * @param {number[]} hueValues - The hue values of the four clusters.
 * @returns {boolean} True if the colors form a square harmony.
 */
function isSquareHarmony(hueValues: number[]): boolean {
    if (hueValues.length !== 4) return false;

    const allDeltas: number[] = [];
    for (let index = 0; index < 4; index++) {
        for (let innerIndex = index + 1; innerIndex < 4; innerIndex++) {
            allDeltas.push(angularHueDifference(hueValues[index], hueValues[innerIndex]));
        }
    }

    const nearNinetyCount = allDeltas.filter(delta => delta >= 70 && delta <= 110).length;
    const nearOneEightyCount = allDeltas.filter(delta => delta >= 160 && delta <= 200).length;

    return nearNinetyCount >= 4 && nearOneEightyCount >= 2;
}

/**
 * Classifies 4+ hue clusters. Falls back to polychromatic.
 *
 * @param {number[]} hueValues - The hue values of the clusters.
 * @returns {HarmonyType} The harmony type.
 */
function classifyFourPlusClusters(hueValues: number[]): HarmonyType {
    if (isSquareHarmony(hueValues)) return 'square';

    const sortedDeltas: number[] = [];
    for (let index = 0; index < hueValues.length; index++) {
        for (let innerIndex = index + 1; innerIndex < hueValues.length; innerIndex++) {
            sortedDeltas.push(angularHueDifference(hueValues[index], hueValues[innerIndex]));
        }
    }
    sortedDeltas.sort((deltaA, deltaB) => deltaA - deltaB);

    const nearRightAngleCount = sortedDeltas.filter(
        delta => (delta >= 70 && delta <= 110) || (delta >= 160 && delta <= 200)
    ).length;
    if (nearRightAngleCount >= sortedDeltas.length * 0.6) return 'tetradic';

    return 'polychromatic';
}

/**
 * Classifies hue clusters into a harmony type based on count.
 *
 * @param {number[]} hueValues - The hue values of the clusters.
 * @returns {HarmonyType} The harmony type.
 */
function classifyHueClusters(hueValues: number[]): HarmonyType {
    if (hueValues.length === 1) return 'monochromatic';
    if (hueValues.length === 2) return classifyTwoClusters(hueValues) ?? 'not_identified';
    if (hueValues.length === 3) return classifyThreeClusters(hueValues) ?? 'not_identified';
    if (hueValues.length >= 4) return classifyFourPlusClusters(hueValues);
    return 'not_identified';
}

/**
 * Detects the color harmony type from pre-computed agglomerative clusters.
 *
 * Uses the same 3D clusters that the distribution bar displays, so the
 * harmony classification always matches the visible color groups.
 *
 * @param clusters - Color clusters from `agglomerativeGrouping()`.
 * @returns The detected harmony type.
 */
export function detectColorHarmony(clusters: ColorCluster[]): HarmonyType {
    if (clusters.length === 0) return 'not_identified';

    const achromaticClusters = clusters.filter(
        cluster => centroidToSaturation(cluster.centroid) < ACHROMATIC_SATURATION_THRESHOLD
    );

    const chromaticClusters = clusters.filter(
        cluster => centroidToSaturation(cluster.centroid) >= MINIMUM_CHROMATIC_SATURATION
    );

    if (achromaticClusters.length === clusters.length) return 'achromatic';
    if (chromaticClusters.length === 0) return 'neutral';

    const hueValues = chromaticClusters.map(cluster => centroidToHue(cluster.centroid));

    return classifyHueClusters(hueValues);
}
