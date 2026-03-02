import { createSignal } from 'solid-js';
import { listen } from '@tauri-apps/api/event';

// Centralized store to track thumbnail regeneration state
// This persists across component mount/unmount cycles (virtualization)

interface ThumbnailReadyPayload {
    id: number;
    path: string;
}

interface RegenerationState {
    pending: Set<number>; // IDs that are waiting for regeneration
    completed: Map<number, string>; // ID -> new thumbnail path
}

const [state, setState] = createSignal<RegenerationState>({
    pending: new Set(),
    completed: new Map()
});

// Subscribers for thumbnail ready events
type ThumbnailCallback = (id: number, path: string) => void;
const subscribers = new Map<number, Set<ThumbnailCallback>>();

// Global listener - initialized once
let listenerInitialized = false;

async function initGlobalListener() {
    if (listenerInitialized) return;
    listenerInitialized = true;

    await listen<ThumbnailReadyPayload>('thumbnail:ready', event => {
        const { id, path } = event.payload;

        // Update store
        markRegenerationComplete(id, path);

        // Notify subscribers
        const callbacks = subscribers.get(id);
        if (callbacks) {
            callbacks.forEach(cb => cb(id, path));
        }
    });
}

// Initialize listener immediately
initGlobalListener();

export function subscribeThumbnailReady(assetId: number, callback: ThumbnailCallback): () => void {
    if (!subscribers.has(assetId)) {
        subscribers.set(assetId, new Set());
    }
    subscribers.get(assetId)!.add(callback);

    // Return unsubscribe function
    return () => {
        const callbacks = subscribers.get(assetId);
        if (callbacks) {
            callbacks.delete(callback);
            if (callbacks.size === 0) {
                subscribers.delete(assetId);
            }
        }
    };
}

export function markPendingRegeneration(assetId: number) {
    setState(s => {
        const newPending = new Set(s.pending);
        newPending.add(assetId);
        return { ...s, pending: newPending };
    });
}

export function markRegenerationComplete(assetId: number, thumbnailPath: string) {
    setState(s => {
        const newPending = new Set(s.pending);
        newPending.delete(assetId);
        const newCompleted = new Map(s.completed);
        newCompleted.set(assetId, thumbnailPath);
        return { pending: newPending, completed: newCompleted };
    });
}

export function isPendingRegeneration(assetId: number): boolean {
    return state().pending.has(assetId);
}

export function getCompletedThumbnail(assetId: number): string | undefined {
    return state().completed.get(assetId);
}

export function clearCompleted(assetId: number) {
    setState(s => {
        const newCompleted = new Map(s.completed);
        newCompleted.delete(assetId);
        return { ...s, completed: newCompleted };
    });
}
