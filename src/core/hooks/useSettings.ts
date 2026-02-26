import {
    thumbnailThreads,
    cacheRetentionDays,
    cacheStats,
    settingsActions
} from '../store/settingsStore';

/**
 * Hook providing access to application settings and management actions.
 * Centralizes state access for settings panels.
 *
 * @returns {Object} Settings state and actions.
 */
export const useSettings = () => {
    return {
        /** Number of worker threads for thumbnail generation */
        thumbnailThreads,
        /** Number of days to keep transcoded files in cache */
        cacheRetentionDays,
        /** Current cache usage statistics */
        cacheStats,

        /** Initialize settings from backend */
        initialize: settingsActions.initialize,
        /** Update one or more settings with validation */
        updateSettings: settingsActions.updateSettings,
        /** Force refresh of cache statistics */
        refreshCacheStats: settingsActions.refreshCacheStats
    };
};
