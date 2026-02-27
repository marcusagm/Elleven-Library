/**
 * ViewportController
 *
 * Domain service that orchestrates communication between the main thread
 * and the layout worker. It translates application state into worker commands
 * and pushes worker results back to the viewportStore.
 */

import { batch } from 'solid-js';
import { unwrap } from 'solid-js/store';
import type {
    LayoutItemInput,
    LayoutConfig,
    ItemPosition,
    IViewportController,
    WorkerOutMessage,
    LayoutMode
} from './types';
import { viewportActions, viewportState } from '../store/viewportStore';
import { scheduler } from '../utils/scheduler';
import {
    LayoutCompleteMessageSchema,
    ErrorMessageSchema,
    PositionResultMessageSchema
} from './schemas';

// Import worker with Vite's native worker support
import LayoutWorker from './layout.worker?worker';

export class ViewportController implements IViewportController {
    private worker: Worker;
    private disposed = false;

    // Debounce resize for smoother performance
    private resizeTimeout: ReturnType<typeof setTimeout> | null = null;

    // Pending position queries
    private pendingQueries = new Map<string, (pos: ItemPosition | null) => void>();

    constructor() {
        this.worker = new LayoutWorker();
        this.setupWorkerListeners();

        // Initialize worker with current store config
        this.postMessage({ type: 'CONFIGURE', payload: unwrap(viewportState.config) });
    }

    /**
     * Public API compatibility (Legacy/Signal transition)
     * These now point to the centralized store.
     */
    get visibleItems(): () => ItemPosition[] {
        return () => viewportState.visibleItems;
    }

    get totalHeight(): () => number {
        return () => viewportState.totalHeight;
    }

    get isCalculating(): () => boolean {
        return () => viewportState.isCalculating;
    }

    /**
     * Sets the items to be laid out.
     */
    setItems(items: LayoutItemInput[]): void {
        if (this.disposed) return;

        batch(() => {
            viewportActions.setItems(items);
            viewportActions.updateFromWorker({ isCalculating: true });
        });

        this.postMessage({ type: 'SET_ITEMS', payload: items });
    }

    /**
     * Updates layout configuration.
     */
    setConfig(config: Partial<LayoutConfig>): void {
        if (this.disposed) return;

        // Update store first
        viewportActions.setConfig(config);
        viewportActions.updateFromWorker({ isCalculating: true });

        // Then notify worker
        this.postMessage({ type: 'CONFIGURE', payload: unwrap(viewportState.config) });
    }

    /**
     * Handles container resize.
     */
    handleResize(width: number): void {
        if (this.disposed) return;
        if (Math.abs(viewportState.config.containerWidth - width) <= 5) return;

        viewportActions.setConfig({ containerWidth: width });

        if (this.resizeTimeout) {
            clearTimeout(this.resizeTimeout);
        }

        this.resizeTimeout = setTimeout(() => {
            if (this.disposed) return;
            this.resizeTimeout = null;
            viewportActions.updateFromWorker({ isCalculating: true });
            this.postMessage({
                type: 'RESIZE',
                payload: { width: viewportState.config.containerWidth }
            });
        }, 50);
    }

    /**
     * Handles scroll position changes.
     */
    handleScroll(scrollTop: number, viewportHeight: number): void {
        if (this.disposed) return;

        // Current scroll task for the scheduler
        const scrollTask = () => {
            if (this.disposed) return;
            this.postMessage({
                type: 'SCROLL',
                payload: { scrollTop, viewportHeight }
            });
        };

        // Update store immediately
        viewportActions.setScroll(scrollTop, viewportHeight);

        // Schedule worker update
        scheduler.schedule(scrollTask);
    }

    /**
     * Queries the exact position of an item from the worker.
     */
    getItemPosition(id: number): Promise<ItemPosition | null> {
        if (this.disposed) return Promise.resolve(null);

        return new Promise(resolve => {
            const requestId = Math.random().toString(36).substring(2, 9);
            this.pendingQueries.set(requestId, resolve);
            this.postMessage({ type: 'QUERY_POSITION', payload: { id, requestId } });

            setTimeout(() => {
                if (this.pendingQueries.has(requestId)) {
                    this.pendingQueries.delete(requestId);
                    resolve(null);
                }
            }, 500);
        });
    }

    /**
     * Cleans up worker and cancels pending operations.
     */
    dispose(): void {
        if (this.disposed) return;
        this.disposed = true;

        if (this.resizeTimeout !== null) {
            clearTimeout(this.resizeTimeout);
        }

        this.worker.terminate();
    }

    private setupWorkerListeners(): void {
        this.worker.onmessage = (e: MessageEvent<WorkerOutMessage>) => {
            if (this.disposed) return;

            const { type, payload } = e.data;

            switch (type) {
                case 'LAYOUT_COMPLETE': {
                    const parsed = LayoutCompleteMessageSchema.parse(e.data);
                    batch(() => {
                        viewportActions.updateFromWorker({
                            totalHeight: parsed.payload.totalHeight,
                            isCalculating: false
                        });
                    });

                    // Re-query visibility with last known scroll
                    this.postMessage({
                        type: 'SCROLL',
                        payload: unwrap(viewportState.scrollPosition)
                    });
                    break;
                }

                case 'VISIBLE_UPDATE': {
                    // Fast path typeguard for 60fps operation
                    if (Array.isArray(payload)) {
                        viewportActions.updateFromWorker({
                            visibleItems: payload as ItemPosition[]
                        });
                    }
                    break;
                }

                case 'ERROR': {
                    const parsed = ErrorMessageSchema.parse(e.data);
                    console.error('[ViewportController] Worker error:', parsed.payload.message);
                    viewportActions.updateFromWorker({ isCalculating: false });
                    break;
                }

                case 'POSITION_RESULT': {
                    const parsed = PositionResultMessageSchema.parse(e.data);
                    const callback = this.pendingQueries.get(parsed.payload.requestId);
                    if (callback) {
                        this.pendingQueries.delete(parsed.payload.requestId);
                        callback(parsed.payload.position);
                    }
                    break;
                }
            }
        };

        this.worker.onerror = error => {
            console.error('[ViewportController] Worker crashed:', error);
            viewportActions.updateFromWorker({ isCalculating: false });
        };
    }

    private postMessage(message: { type: string; payload?: unknown }): void {
        if (!this.disposed) {
            this.worker.postMessage(message);
        }
    }
}

// ============================================================================
// Factory Function
// ============================================================================

/**
 * Creates a new ViewportController instance.
 */
export function createViewportController(
    mode: LayoutMode = 'masonry',
    initialConfig: Partial<LayoutConfig> = {}
): ViewportController {
    const controller = new ViewportController();
    // Use timeout or next tick? The constructor sends initial config, we can send mode right after.
    controller.setConfig({ mode, ...initialConfig });
    return controller;
}
