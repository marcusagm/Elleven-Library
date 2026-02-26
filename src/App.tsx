import { onMount, onCleanup, Show, createEffect, createMemo } from 'solid-js';
import {
    useSystem,
    useNotification,
    useSelection,
    useLibrary,
    useMetadataNotifications
} from './core/hooks';
import { AppShell } from './layouts/AppShell';
import { LibrarySidebar } from './components/layout/LibrarySidebar';
import { FileInspector } from './components/layout/FileInspector';
import { GlobalStatusbar } from './components/layout/GlobalStatusbar';
import { Viewport } from './components/layout/Viewport';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
// Native DnD
import {
    dndRegistry,
    TagDropStrategy,
    ImageDropStrategy,
    currentDragItem,
    setDropTargetId
} from './core/dnd';
import { Sonner } from './components/ui';
import { Loader } from './components/ui/Loader';
import { SettingsModal } from './components/features/settings';
// Input System
import { InputProvider, useShortcuts } from './core/input';
import logoColor from './assets/logo-color.svg';
import logoWhite from './assets/logo-white.svg';
import { appearance } from './core/store/appearanceStore';

/**
 * Main application component.
 * Manages global initialization, shortcuts, and the top-level layout.
 *
 * @returns {JSX.Element} The application layout.
 */
function App() {
    const system = useSystem();
    const notification = useNotification();
    const selection = useSelection();
    const lib = useLibrary();

    // Start background sync notifications
    useMetadataNotifications();

    const effectiveLogo = createMemo(() => {
        let mode = appearance().mode;
        if (mode === 'system') {
            mode = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        return mode === 'dark' ? logoWhite : logoColor;
    });

    // Global shortcuts via Input Service
    useShortcuts([
        {
            keys: 'Meta+Comma',
            name: 'Settings',
            action: () => system.openSettings(true)
        },
        {
            keys: 'Meta+KeyA',
            name: 'Select All',
            ignoreInputs: true,
            action: () => {
                const allIds = lib.items.map(i => i.id);
                selection.select(allIds);
            }
        },
        {
            keys: 'Escape',
            name: 'Deselect All',
            action: () => {
                const active = document.activeElement;
                if (active && ['INPUT', 'TEXTAREA'].includes(active.tagName)) {
                    (active as HTMLElement).blur();
                } else {
                    selection.select([]);
                }
            }
        }
    ]);

    // Root-level DND cleanup
    createEffect(() => {
        if (!currentDragItem()) {
            setDropTargetId(null);
        }
    });

    onMount(() => {
        system.initialize();
        import('./core/store/appearanceStore').then(({ appearanceActions }) => {
            appearanceActions.initialize();
        });
        import('./core/store/formatStore').then(({ formatActions }) => {
            formatActions.initialize();
        });

        // Register Strategies
        dndRegistry.register('TAG', TagDropStrategy);
        dndRegistry.register('IMAGE', ImageDropStrategy);

        // Listen for indexing completion (with proper cleanup)
        let unlistenIndexerComplete: (() => void) | null = null;
        listen('indexer:complete', () => {
            notification.success('Indexing Complete', 'Library update finished');
        }).then(unlisten => {
            unlistenIndexerComplete = unlisten;
        });

        onCleanup(() => {
            if (unlistenIndexerComplete) unlistenIndexerComplete();
        });

        // Notify Splash Screen
        window.dispatchEvent(new CustomEvent('app-ready'));
    });

    const handleSelectFolder = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Select Reference Library Folder'
            });

            if (selected) {
                const path = typeof selected === 'string' ? selected : String(selected);
                if (path) {
                    notification.info(
                        'Indexing Started',
                        `Processing folder: ${path.split(/[\\/]/).pop()}`
                    );
                    await system.setRootLocation(path);
                }
            }
        } catch (err) {
            console.error('Failed to select folder:', err);
        }
    };

    return (
        <Show
            when={!system.loading()}
            fallback={<Loader isFullscreen text="Initializing Mundam..." />}
        >
            <Show
                when={system.rootPath()}
                fallback={
                    <div class="welcome-screen">
                        <img src={effectiveLogo()} alt="Mundam Logo" class="welcome-logo" />
                        {/* <h1>Mundam</h1> */}
                        <p>Start by choosing a folder to monitor for visual references.</p>
                        <button class="primary-btn" onClick={handleSelectFolder}>
                            Initialize Library
                        </button>
                    </div>
                }
            >
                <AppShell
                    sidebar={<LibrarySidebar />}
                    inspector={<FileInspector />}
                    statusbar={<GlobalStatusbar />}
                >
                    <Viewport />
                </AppShell>
                <Sonner position="bottom-right" useRichColors />
                <SettingsModal
                    isOpen={system.isSettingsOpen()}
                    onClose={() => system.openSettings(false)}
                    initialTab="general"
                />
            </Show>
        </Show>
    );
}

/**
 * Root application wrapper with context providers.
 *
 * @returns {JSX.Element} The application with providers.
 */
function AppWithProvider() {
    return (
        <InputProvider>
            <App />
        </InputProvider>
    );
}

export default AppWithProvider;
