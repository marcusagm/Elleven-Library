import { createSignal } from 'solid-js';
import { type FileFormat } from './formatStore';
import { listen } from '@tauri-apps/api/event';
import { addLocation, initDb } from '../../lib/db';
import { tauriService } from '../tauri/services';
import { metadataActions } from './metadata';
import { type BatchChangePayload } from './library';
import { initStreamingToken } from '../../lib/hls-player';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export interface ProgressPayload {
    total: number;
    processed: number;
    current_file: string;
}

const [loading, setLoading] = createSignal(true);
const [progress, setProgress] = createSignal<ProgressPayload | null>(null);
const [thumbnailProgress, setThumbnailProgress] = createSignal<ProgressPayload | null>(null);
const [rootPath, setRootPath] = createSignal<string | null>(null);
const [initialized, setInitialized] = createSignal(false);
const [supportedFormats, setSupportedFormats] = createSignal<FileFormat[]>([]);
const [isSettingsOpen, setIsSettingsOpen] = createSignal(false);
const [isDesignSystemOpen, setIsDesignSystemOpen] = createSignal(false);

export const systemActions = {
    initialize: async () => {
        if (initialized()) return;

        try {
            setLoading(true);
            await initDb();
            await initStreamingToken();
            await metadataActions.loadLocations();
            await metadataActions.loadTags();
            await metadataActions.loadSmartFolders();

            const formats = await tauriService.getLibrarySupportedFormats();
            setSupportedFormats(formats);

            // Auto-select root path if locations exist
            import('./metadata').then(({ metadataState }) => {
                if (metadataState.locations.length > 0) {
                    const main = metadataState.locations[0];
                    setRootPath(main.path);
                    // Trigger initial load
                    import('./library').then(({ libraryActions }) => {
                        libraryActions.refreshImages(true);
                    });
                    metadataActions.loadStats();
                }
            });

            // Setup Listeners
            listen<ProgressPayload>('indexer:progress', e => {
                systemActions.updateProgress(e.payload);
            });

            listen<number>('indexer:complete', () => {
                systemActions.clearProgress();
                // Refresh library to show new items
                // We use a small delay or just call it directly
                // Importing actions inside function to avoid potential circular dependency issues,
                // though we are importing them at top level.
                // Circular dependency libraryStore <-> systemStore might exist if not careful.
                // libraryStore imports systemStore? Previous check showed unused import removed.
                // libraryStore DOES NOT import systemStore anymore.
                // systemStore imports metadata.
                // We need libraryActions here.
                import('./library').then(({ libraryActions }) => {
                    libraryActions.refreshImages(true);
                });
                metadataActions.loadStats();
                metadataActions.loadLocations();
            });

            listen<{ id: number; path: string }>('thumbnail:ready', e => {
                import('./library').then(({ libraryActions }) => {
                    libraryActions.updateThumbnail(e.payload.id, e.payload.path);
                });
            });

            listen<ProgressPayload>('thumbnail:queue-status', e => {
                systemActions.updateThumbnailProgress(e.payload);
            });

            listen<BatchChangePayload>('library:batch-change', e => {
                const payload = e.payload;

                import('./library').then(({ libraryActions }) => {
                    libraryActions.handleBatchChange(payload);
                });

                // Also update stats
                import('./metadata').then(({ metadataActions }) => {
                    metadataActions.handleBatchChange(payload);
                });
            });

            setInitialized(true);
        } catch (err) {
            console.error('Initialization failed:', err);
        } finally {
            setLoading(false);
        }
    },

    setRootLocation: async (path: string) => {
        await addLocation(path);
        await metadataActions.loadLocations();
        setRootPath(path);
        await tauriService.startIndexing({ path });
    },

    updateProgress: (payload: ProgressPayload) => {
        setProgress(payload);
    },

    clearProgress: () => {
        setProgress(null);
    },

    setRootPath: (path: string | null) => {
        setRootPath(path);
    },

    setLoading: (isLoading: boolean) => {
        setLoading(isLoading);
    },

    updateThumbnailProgress: (payload: ProgressPayload) => {
        setThumbnailProgress(payload);
    },

    openSettings: (open = true) => {
        setIsSettingsOpen(open);
    },

    openDesignSystem: async (open = true) => {
        if (!open) {
            setIsDesignSystemOpen(false);
            return;
        }

        try {
            const label = 'design-system';
            const existing = await WebviewWindow.getByLabel(label);
            if (existing) {
                await existing.setFocus();
                return;
            }

            const webview = new WebviewWindow(label, {
                url: 'index.html#design-system',
                title: 'Mundam Design System',
                width: 1200,
                height: 900
            });

            webview.once('tauri://error', error => {
                console.error('Webview error:', error);
            });

            setIsDesignSystemOpen(true);
        } catch (error) {
            console.error('Failed to open design system window:', error);
        }
    },

    runDbMaintenance: async () => {
        await tauriService.runDbMaintenance();
    },

    cleanupCache: async (maxAgeDays?: number) => {
        return await tauriService.cleanupCache(maxAgeDays);
    },

    clearCache: async () => {
        return await tauriService.clearCache();
    }
};

export {
    loading,
    progress,
    thumbnailProgress,
    rootPath,
    supportedFormats,
    isSettingsOpen,
    isDesignSystemOpen
};
