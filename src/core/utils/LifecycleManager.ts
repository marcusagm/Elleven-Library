/**
 * @module LifecycleManager
 * @description
 * Centralized lifecycle management for Solid.js components receiving Rust (Tauri IPC) events,
 * and Telemetry Bridge wrapper to forward frontend traces to the centralized `tracing` backend log.
 */
import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
import { onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

export const LifecycleManager = {
    /**
     * Sends structured telemetry logs to the backend via IPC to unify distributed traces.
     * Use this instead of `console.log` for any operations related to IO, rendering speed, or IPC interactions.
     *
     * @param level - The severity of the log.
     * @param component - The logical module reporting the log (e.g., 'VirtualListView', 'FileWatcherHandler').
     * @param message - The descriptive log trace.
     */
    logTelemetry: async (
        level: 'info' | 'warn' | 'error' | 'debug',
        component: string,
        message: string
    ): Promise<void> => {
        try {
            await invoke('send_telemetry_log', { level, component, message });
        } catch (e) {
            console.error('Failed to send telemetry log', e);
        }
    },

    /**
     * Subscribes to a Tauri IPC event and automatically registers the unlisten
     * cleanup function within the current Solid.js reactive context.
     *
     * **Critical Architectural Rule**: Never call `listen()` directly inside generic UI components without this,
     * to prevent memory leaks during component unmount.
     *
     * @param eventName - The exact string identifier of the event broadcasted by the rust backend.
     * @param handler - The typed callback handling the `Event.payload`.
     */
    registerListener: <T>(eventName: string, handler: (payload: T) => void): void => {
        let unlistenFunction: UnlistenFn | null = null;
        let isCleanedUp = false;

        listen<T>(eventName, (event: Event<T>) => {
            handler(event.payload);
        })
            .then(unlistener => {
                // It's possible the component unmounted *before* listen() resolved
                if (isCleanedUp) {
                    unlistener();
                } else {
                    unlistenFunction = unlistener;
                }
            })
            .catch((error: unknown) => {
                LifecycleManager.logTelemetry(
                    'error',
                    'LifecycleManager',
                    `Failed to register listener for ${eventName}: ${String(error)}`
                );
            });

        onCleanup(() => {
            isCleanedUp = true;
            if (unlistenFunction) {
                unlistenFunction();
                LifecycleManager.logTelemetry(
                    'debug',
                    'LifecycleManager',
                    `Unlistened event: ${eventName}`
                );
            }
        });
    }
};
