import { Component, createSignal, onMount, onCleanup, Show } from 'solid-js';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { detectPlatform } from '../../../core/input/utils/platform';

/**
 * Native-style window control buttons (minimize, maximize/restore, close).
 * Renders only on Windows and Linux where the native title bar is removed.
 * On macOS, the native traffic lights are preserved via `hiddenTitle` and `decorations: false`
 * with `titleBarStyle: "overlay"` in the Tauri config, so this component renders nothing.
 *
 * @returns {JSX.Element} The rendered window controls or an empty fragment on macOS.
 */
export const WindowControls: Component = () => {
    const platform = detectPlatform();
    const [isMaximized, setIsMaximized] = createSignal(false);

    onMount(async () => {
        if (platform === 'mac') return;

        const applicationWindow = getCurrentWindow();

        await applicationWindow.setDecorations(false);

        setIsMaximized(await applicationWindow.isMaximized());

        const unlistenResize = await applicationWindow.onResized(async () => {
            setIsMaximized(await applicationWindow.isMaximized());
        });

        onCleanup(() => {
            unlistenResize();
        });
    });

    /**
     * Minimizes the application window.
     *
     * @returns {Promise<void>}
     */
    const handleMinimize = async (): Promise<void> => {
        await getCurrentWindow().minimize();
    };

    /**
     * Toggles between maximized and restored window state.
     *
     * @returns {Promise<void>}
     */
    const handleToggleMaximize = async (): Promise<void> => {
        await getCurrentWindow().toggleMaximize();
    };

    /**
     * Closes the application window.
     *
     * @returns {Promise<void>}
     */
    const handleClose = async (): Promise<void> => {
        await getCurrentWindow().close();
    };

    return (
        <Show when={platform !== 'mac'}>
            <div class="titlebar-window-controls">
                <button
                    class="titlebar-window-control-button"
                    aria-label="Minimize window"
                    onClick={handleMinimize}
                >
                    <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
                        <rect width="10" height="1" />
                    </svg>
                </button>

                <button
                    class="titlebar-window-control-button"
                    aria-label={isMaximized() ? 'Restore window' : 'Maximize window'}
                    onClick={handleToggleMaximize}
                >
                    <Show
                        when={isMaximized()}
                        fallback={
                            <svg
                                width="10"
                                height="10"
                                viewBox="0 0 10 10"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1"
                            >
                                <rect x="0.5" y="0.5" width="9" height="9" />
                            </svg>
                        }
                    >
                        <svg
                            width="10"
                            height="10"
                            viewBox="0 0 10 10"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="1"
                        >
                            <rect x="2.5" y="0.5" width="7" height="7" />
                            <rect x="0.5" y="2.5" width="7" height="7" />
                        </svg>
                    </Show>
                </button>

                <button
                    class="titlebar-window-control-button titlebar-window-control-close"
                    aria-label="Close window"
                    onClick={handleClose}
                >
                    <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
                        <path d="M1.7 0.3L0.3 1.7L3.6 5L0.3 8.3L1.7 9.7L5 6.4L8.3 9.7L9.7 8.3L6.4 5L9.7 1.7L8.3 0.3L5 3.6L1.7 0.3Z" />
                    </svg>
                </button>
            </div>
        </Show>
    );
};
