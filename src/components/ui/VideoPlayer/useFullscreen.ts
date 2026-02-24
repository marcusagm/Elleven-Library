import { createSignal, onMount, onCleanup } from 'solid-js';
import { getCurrentWindow } from '@tauri-apps/api/window';

/**
 * Custom hook to manage the fullscreen state of the video player.
 * It integrates natively with Tauri's window API for desktop environments,
 * while safely falling back to CSS fixed positioning if the API is restricted.
 *
 * @param containerElementAccessor - Accessor for the main video player container `HTMLDivElement`
 * @returns An object containing the current `isFullscreen` state and the `toggleFullscreen` action
 *
 * @example
 * ```tsx
 * const { isFullscreen, toggleFullscreen } = useFullscreen(containerRef);
 * ```
 */
export function useFullscreen() {
    const [isFullscreen, setIsFullscreen] = createSignal(false);
    let didForceFullscreen = false;

    /**
     * Toggles the fullscreen state using Tauri's window API.
     * Also updates the reactive signal, ensuring CSS fallback styles apply instantly.
     *
     * @param event - Optional mouse event from the trigger button
     */
    const toggleFullscreen = async (event?: MouseEvent) => {
        if (event) {
            event.stopPropagation();
        }

        const nextState = !isFullscreen();
        setIsFullscreen(nextState);
        didForceFullscreen = nextState;

        try {
            const appWindow = getCurrentWindow();
            await appWindow.setFullscreen(nextState);
        } catch (error) {
            console.error('Failed to toggle fullscreen in Tauri app window:', error);
        }
    };

    onMount(() => {
        let unlistenResize: (() => void) | null = null;

        try {
            const appWindow = getCurrentWindow();

            appWindow
                .onResized(() => {
                    appWindow
                        .isFullscreen()
                        .then((isCurrentlyFull: boolean) => {
                            setIsFullscreen(isCurrentlyFull);
                        })
                        .catch(() => {});
                })
                .then((unlistenFn: () => void) => {
                    unlistenResize = unlistenFn;
                })
                .catch(() => {});
        } catch {
            console.warn('Tauri window API is not available, falling back to CSS-only fullscreen.');
        }

        onCleanup(() => {
            if (unlistenResize) {
                unlistenResize();
            }

            if (didForceFullscreen) {
                try {
                    const appWindow = getCurrentWindow();
                    appWindow.setFullscreen(false).catch(() => {});
                } catch {
                    // Ignore
                }
            }
        });
    });

    return {
        isFullscreen,
        toggleFullscreen
    };
}
