import { metadataState, setMetadataState } from './metadataState';
import { getLocations } from '../../../lib/db';
import { tagService } from '../../../lib/tags';
import { computeStatsFromBatchChange } from '../statsHelpers';
import { type BatchChangePayload } from '../library';

/** Check if some added items belong to unknown folders */
function hasUnknownFolders(added: BatchChangePayload['added'], knownIds: Set<number>): boolean {
    if (!added) return false;
    return added.some(item => item.folder_id && !knownIds.has(item.folder_id));
}

// Minimal definition of refreshAll to avoid circular dependency
let tagRefs = { loadTags: async () => {} };
let searchRefs = { loadSmartFolders: async () => {} };
export function initLocationRefs(
    tags: { loadTags: () => Promise<void> },
    searches: { loadSmartFolders: () => Promise<void> }
) {
    tagRefs = tags;
    searchRefs = searches;
}

export const locationActions = {
    loadLocations: async () => {
        try {
            const locations = await getLocations();
            setMetadataState('locations', locations);
        } catch (error) {
            console.error('Failed to load locations:', error);
        }
    },

    loadStats: async () => {
        try {
            const stats = await tagService.getLibraryStats();
            const tagMap = new Map();
            stats.tag_counts.forEach(c => tagMap.set(c.tag_id, c.count));

            const folderMap = new Map();
            stats.folder_counts.forEach(c => folderMap.set(c.folder_id, c.count));

            const folderRecursiveMap = new Map();
            if (stats.folder_counts_recursive) {
                stats.folder_counts_recursive.forEach(c =>
                    folderRecursiveMap.set(c.folder_id, c.count)
                );
            }

            setMetadataState('libraryStats', {
                total_assets: stats.total_assets,
                untagged_assets: stats.untagged_assets,
                tag_counts: tagMap,
                folder_counts: folderMap,
                folder_counts_recursive: folderRecursiveMap
            });
        } catch (error) {
            console.error('Failed to load library stats:', error);
        }
    },

    refreshAll: async () => {
        await Promise.all([
            tagRefs.loadTags(),
            locationActions.loadLocations(),
            locationActions.loadStats(),
            searchRefs.loadSmartFolders()
        ]);
    },

    handleBatchChange: (payload: BatchChangePayload) => {
        const knownIds = new Set(metadataState.locations.map(location => location.id));
        let needsRefresh = payload.needs_refresh ?? false;

        if (hasUnknownFolders(payload.added, knownIds)) {
            needsRefresh = true;
        }

        setMetadataState('libraryStats', stats => {
            const result = computeStatsFromBatchChange(
                stats,
                payload,
                metadataState.locations,
                knownIds
            );
            needsRefresh = needsRefresh || result.needsRefresh;
            return result.newStats;
        });

        if (needsRefresh) {
            locationActions.refreshAll();
        }
    }
};
