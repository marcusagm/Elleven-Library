/**
 * Color clustering utilities using agglomerative hierarchical clustering
 * in cylindrical HSL 3D space.
 *
 * Converts hex colors to HSL, maps to 3D Cartesian coordinates, and
 * performs bottom-up clustering to group visually similar colors into
 * 3-5 meaningful families.
 *
 * Shared by `ColorDistribution` (bar) and `detectColorHarmony` (badge).
 */

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

/** A 3D point in the cylindrical HSL Cartesian space. */
export interface CartesianColorPoint {
    coordinateX: number;
    coordinateY: number;
    coordinateZ: number;
}

/** A cluster produced by agglomerative grouping. */
export interface ColorCluster {
    /** Weighted centroid in 3D cylindrical HSL space. */
    centroid: CartesianColorPoint;
    /** Total percentage this cluster represents (sum of all member colors). */
    totalPercentage: number;
    /** Hex of the single most dominant color in this cluster. */
    representativeHex: string;
    /** Percentage of the single most dominant color (for representative selection). */
    highestIndividualPercentage: number;
}

/** Target number of color groups (3–5). */
const MINIMUM_GROUP_COUNT = 3;
const MAXIMUM_GROUP_COUNT = 5;

/**
 * Minimum 3D distance threshold to stop merging groups.
 * Empirically tuned: ~0.35 in the normalized cylindrical HSL space.
 */
const MERGE_DISTANCE_THRESHOLD = 0.35;

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
 * Converts HSL color to 3D Cartesian coordinates in cylindrical space.
 * x = S * cos(H_rad), y = S * sin(H_rad), z = L
 */
function hslToCartesian(hue: number, saturation: number, lightness: number): CartesianColorPoint {
    const hueRadians = (hue * Math.PI) / 180;
    return {
        coordinateX: saturation * Math.cos(hueRadians),
        coordinateY: saturation * Math.sin(hueRadians),
        coordinateZ: lightness
    };
}

/**
 * Calculates Euclidean distance between two 3D Cartesian points.
 */
function euclideanDistance(pointA: CartesianColorPoint, pointB: CartesianColorPoint): number {
    const deltaX = pointA.coordinateX - pointB.coordinateX;
    const deltaY = pointA.coordinateY - pointB.coordinateY;
    const deltaZ = pointA.coordinateZ - pointB.coordinateZ;
    return Math.sqrt(deltaX * deltaX + deltaY * deltaY + deltaZ * deltaZ);
}

/**
 * Computes the weighted centroid of two clusters being merged.
 */
function mergeClusterCentroids(
    clusterA: ColorCluster,
    clusterB: ColorCluster
): CartesianColorPoint {
    const totalWeight = clusterA.totalPercentage + clusterB.totalPercentage;
    if (totalWeight === 0) return clusterA.centroid;

    const weightA = clusterA.totalPercentage / totalWeight;
    const weightB = clusterB.totalPercentage / totalWeight;

    return {
        coordinateX:
            clusterA.centroid.coordinateX * weightA + clusterB.centroid.coordinateX * weightB,
        coordinateY:
            clusterA.centroid.coordinateY * weightA + clusterB.centroid.coordinateY * weightB,
        coordinateZ:
            clusterA.centroid.coordinateZ * weightA + clusterB.centroid.coordinateZ * weightB
    };
}

/**
 * Finds the pair of clusters with the smallest distance.
 */
function findClosestClusterPair(
    clusters: ColorCluster[]
): { indexA: number; indexB: number; distance: number } | null {
    if (clusters.length < 2) return null;

    let minimumDistance = Infinity;
    let closestIndexA = 0;
    let closestIndexB = 1;

    for (let outerIndex = 0; outerIndex < clusters.length; outerIndex++) {
        for (let innerIndex = outerIndex + 1; innerIndex < clusters.length; innerIndex++) {
            const distance = euclideanDistance(
                clusters[outerIndex].centroid,
                clusters[innerIndex].centroid
            );
            if (distance < minimumDistance) {
                minimumDistance = distance;
                closestIndexA = outerIndex;
                closestIndexB = innerIndex;
            }
        }
    }

    return { indexA: closestIndexA, indexB: closestIndexB, distance: minimumDistance };
}

/**
 * Groups colors using agglomerative (bottom-up) hierarchical clustering
 * in the cylindrical HSL Cartesian space.
 *
 * Algorithm:
 * 1. Each color starts as its own cluster
 * 2. Repeatedly find and merge the two closest clusters (by centroid distance)
 * 3. Stop when cluster count is between 3–5, or closest pair exceeds threshold
 *
 * @param colors - All extracted colors from the palette.
 * @returns Array of final color groups sorted by percentage descending.
 */
export function agglomerativeGrouping(colors: ExtractedColorData[]): ColorCluster[] {
    if (colors.length === 0) return [];

    const clusters: ColorCluster[] = colors.map(color => {
        const hsl = hexToHsl(color.hex_color);
        return {
            centroid: hslToCartesian(hsl.hue, hsl.saturation, hsl.lightness),
            totalPercentage: color.percentage,
            representativeHex: color.hex_color,
            highestIndividualPercentage: color.percentage
        };
    });

    while (clusters.length > MINIMUM_GROUP_COUNT) {
        const closestPair = findClosestClusterPair(clusters);
        if (!closestPair) break;

        if (
            clusters.length <= MAXIMUM_GROUP_COUNT &&
            closestPair.distance > MERGE_DISTANCE_THRESHOLD
        ) {
            break;
        }

        const clusterA = clusters[closestPair.indexA];
        const clusterB = clusters[closestPair.indexB];

        const mergedCluster: ColorCluster = {
            centroid: mergeClusterCentroids(clusterA, clusterB),
            totalPercentage: clusterA.totalPercentage + clusterB.totalPercentage,
            representativeHex:
                clusterA.highestIndividualPercentage >= clusterB.highestIndividualPercentage
                    ? clusterA.representativeHex
                    : clusterB.representativeHex,
            highestIndividualPercentage: Math.max(
                clusterA.highestIndividualPercentage,
                clusterB.highestIndividualPercentage
            )
        };

        const higherIndex = Math.max(closestPair.indexA, closestPair.indexB);
        const lowerIndex = Math.min(closestPair.indexA, closestPair.indexB);
        clusters.splice(higherIndex, 1);
        clusters.splice(lowerIndex, 1);
        clusters.push(mergedCluster);
    }

    clusters.sort((groupA, groupB) => groupB.totalPercentage - groupA.totalPercentage);

    return clusters;
}
