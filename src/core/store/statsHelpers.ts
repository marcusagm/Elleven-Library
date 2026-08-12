/**
 * Statistics computation helpers for BatchChangePayload processing.
 *
 * Pure functions that compute library stats deltas from batch change events.
 */

import {
    type BatchChangePayload,
    type BatchChangeAddedItem,
    type BatchChangeRemovedItem
} from './library';

interface FolderNode {
    id: string;
    path: string;
    name: string;
    parent_id: string | null;
    is_root: boolean;
}

export interface StatsSnapshot {
    total_assets: number;
    untagged_assets: number;
    has_tags_assets: number;
    favorite_assets: number;
    trash_assets: number;
    tag_counts: Map<string, number>;
    folder_counts: Map<string, number>;
    folder_counts_recursive: Map<string, number>;
}

/** Get all ancestor folder IDs for the given folder */
function getAncestors(folderId: string, locations: FolderNode[]): string[] {
    const ancestors: string[] = [];
    let currentId: string | null = folderId;
    while (currentId) {
        ancestors.push(currentId);
        const node = locations.find(location => location.id === currentId);
        currentId = node ? node.parent_id : null;
    }
    return ancestors;
}

/** Decrement a map counter, flooring at 0 */
function decrementCounter(counterMap: Map<string, number>, key: string): void {
    const current = counterMap.get(key) || 0;
    if (current > 0) counterMap.set(key, current - 1);
}

/** Increment a map counter */
function incrementCounter(counterMap: Map<string, number>, key: string): void {
    const current = counterMap.get(key) || 0;
    counterMap.set(key, current + 1);
}

/** Apply removals to the stats snapshot */
function applyRemovals(
    removed: BatchChangeRemovedItem[],
    tagCounts: Map<string, number>,
    folderCounts: Map<string, number>,
    folderRecursive: Map<string, number>,
    locations: FolderNode[]
): { totalDiff: number; untaggedDiff: number } {
    let totalDiff = 0;
    let untaggedDiff = 0;

    for (const item of removed) {
        totalDiff--;
        if (!item.tag_ids || item.tag_ids.length === 0) {
            untaggedDiff--;
        } else {
            for (const tagId of item.tag_ids) {
                decrementCounter(tagCounts, tagId);
            }
        }

        if (item.folder_id) {
            decrementCounter(folderCounts, item.folder_id);
            const ancestors = getAncestors(item.folder_id, locations);
            for (const ancestorId of ancestors) {
                decrementCounter(folderRecursive, ancestorId);
            }
        }
    }

    return { totalDiff, untaggedDiff };
}

/** Apply additions to the stats snapshot */
function applyAdditions(
    added: BatchChangeAddedItem[],
    folderCounts: Map<string, number>,
    folderRecursive: Map<string, number>,
    locations: FolderNode[]
): { totalDiff: number; untaggedDiff: number } {
    let totalDiff = 0;
    let untaggedDiff = 0;

    for (const item of added) {
        totalDiff++;
        untaggedDiff++;

        if (item.folder_id) {
            incrementCounter(folderCounts, item.folder_id);
            const ancestors = getAncestors(item.folder_id, locations);
            for (const ancestorId of ancestors) {
                incrementCounter(folderRecursive, ancestorId);
            }
        }
    }

    return { totalDiff, untaggedDiff };
}

/** Apply folder move updates to the stats snapshot */
function applyUpdates(
    updated: BatchChangeAddedItem[],
    folderCounts: Map<string, number>,
    folderRecursive: Map<string, number>,
    locations: FolderNode[],
    knownIds: Set<string>
): boolean {
    let needsRefresh = false;

    for (const item of updated) {
        if (item.old_folder_id && item.old_folder_id !== item.folder_id) {
            decrementCounter(folderCounts, item.old_folder_id);
            const oldAncestors = getAncestors(item.old_folder_id, locations);
            for (const ancestorId of oldAncestors) {
                decrementCounter(folderRecursive, ancestorId);
            }

            incrementCounter(folderCounts, item.folder_id);
            const newAncestors = getAncestors(item.folder_id, locations);
            for (const ancestorId of newAncestors) {
                incrementCounter(folderRecursive, ancestorId);
            }
        }

        if (item.folder_id && !knownIds.has(item.folder_id)) {
            needsRefresh = true;
        }
    }

    return needsRefresh;
}

/** Compute the new stats snapshot from a batch change payload */
export function computeStatsFromBatchChange(
    currentStats: StatsSnapshot,
    payload: BatchChangePayload,
    locations: FolderNode[],
    knownIds: Set<string>
): { newStats: StatsSnapshot; needsRefresh: boolean } {
    const tagCounts = new Map(currentStats.tag_counts);
    const folderCounts = new Map(currentStats.folder_counts);
    const folderRecursive = new Map(currentStats.folder_counts_recursive);

    let totalDiff = 0;
    let untaggedDiff = 0;
    let needsRefresh = false;

    if (payload.removed) {
        const result = applyRemovals(
            payload.removed,
            tagCounts,
            folderCounts,
            folderRecursive,
            locations
        );
        totalDiff += result.totalDiff;
        untaggedDiff += result.untaggedDiff;
    }

    if (payload.added) {
        const result = applyAdditions(payload.added, folderCounts, folderRecursive, locations);
        totalDiff += result.totalDiff;
        untaggedDiff += result.untaggedDiff;
    }

    if (payload.updated) {
        needsRefresh = applyUpdates(
            payload.updated,
            folderCounts,
            folderRecursive,
            locations,
            knownIds
        );
    }

    return {
        newStats: {
            total_assets: currentStats.total_assets + totalDiff,
            untagged_assets: currentStats.untagged_assets + untaggedDiff,
            has_tags_assets: currentStats.has_tags_assets, // Approximated
            favorite_assets: currentStats.favorite_assets, // Approximated
            trash_assets: currentStats.trash_assets, // Approximated
            tag_counts: tagCounts,
            folder_counts: folderCounts,
            folder_counts_recursive: folderRecursive
        },
        needsRefresh
    };
}
