import { invoke } from '@tauri-apps/api/core';
import { type FileFormat } from '../store/formatStore';
import { LifecycleManager } from '../utils/LifecycleManager';

// Define strict types for Tauri commands
export interface StartIndexingArgs {
    path: string;
}

export const tauriService = {
    /**
     * Starts the background indexing process for the given directory path.
     * This triggers the 'indexer:progress' and 'indexer:complete' events.
     */
    startIndexing: async (args: StartIndexingArgs): Promise<void> => {
        try {
            await invoke('start_indexing', { path: args.path });
            LifecycleManager.logTelemetry(
                'info',
                'tauriService',
                `Started indexing for: ${args.path}`
            );
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Command 'start_indexing' failed: ${String(error)}`
            );
            throw error;
        }
    },

    /**
     * Example wrapper for other commands...
     */
    // stopIndexing: async () => invoke("stop_indexing"),

    getLibrarySupportedFormats: async (): Promise<FileFormat[]> => {
        try {
            return await invoke('get_library_supported_formats');
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to load formats: ${String(error)}`
            );
            return [];
        }
    },

    runDbMaintenance: async (): Promise<void> => {
        try {
            await invoke('run_db_maintenance');
            LifecycleManager.logTelemetry(
                'info',
                'tauriService',
                'DB Maintenance completed successfully'
            );
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to run DB maintenance: ${String(error)}`
            );
            throw error;
        }
    },

    getSetting: async (key: string): Promise<string | null> => {
        try {
            return await invoke('get_setting', { key });
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to get setting ${key}: ${String(error)}`
            );
            return null;
        }
    },

    setSetting: async (key: string, value: string): Promise<void> => {
        try {
            await invoke('set_setting', { key, value });
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to set setting ${key}: ${String(error)}`
            );
            throw error;
        }
    },

    // --- Cache Management ---

    getCacheStats: async (): Promise<{
        directory: string;
        size_bytes: number;
        file_count: number;
    }> => {
        try {
            return await invoke('get_cache_stats');
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to get cache stats: ${String(error)}`
            );
            return { directory: '', size_bytes: 0, file_count: 0 };
        }
    },

    cleanupCache: async (maxAgeDays?: number): Promise<number> => {
        try {
            const result: number = await invoke('cleanup_cache', { maxAgeDays });
            LifecycleManager.logTelemetry(
                'info',
                'tauriService',
                `Cache cleaned. Cleared ${result} entries.`
            );
            return result;
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to cleanup cache: ${String(error)}`
            );
            throw error;
        }
    },

    clearCache: async (): Promise<number> => {
        try {
            const result: number = await invoke('clear_cache');
            LifecycleManager.logTelemetry(
                'info',
                'tauriService',
                `Cache entirely cleared. Removed ${result} entries.`
            );
            return result;
        } catch (error) {
            LifecycleManager.logTelemetry(
                'error',
                'tauriService',
                `Failed to clear cache: ${String(error)}`
            );
            throw error;
        }
    }
};
