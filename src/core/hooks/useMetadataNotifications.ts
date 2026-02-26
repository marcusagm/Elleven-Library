import { onCleanup, onMount } from 'solid-js';
import { listen } from '@tauri-apps/api/event';
import { useNotification } from './useNotification';
import { type BatchChangePayload } from '../store/libraryStore';

/**
 * Hook that listens for background library changes and triggers notifications.
 * This decouples core state management from transient UI feedback (toasts).
 */
export const useMetadataNotifications = () => {
    const notification = useNotification();

    onMount(() => {
        let unlistenBatchChange: (() => void) | null = null;

        // Set up listener for library batch changes (emitted by backend/indexing)
        listen<BatchChangePayload>('library:batch-change', event => {
            const payload = event.payload;
            const addedCount = payload.added?.length || 0;
            const removedCount = payload.removed?.length || 0;
            const updatedCount = payload.updated?.length || 0;

            if (addedCount > 0) {
                notification.success(
                    'Library Sync',
                    addedCount === 1 ? '1 image added' : `${addedCount} images added`
                );
            }

            if (removedCount > 0) {
                notification.info(
                    'Library Sync',
                    removedCount === 1 ? '1 image removed' : `${removedCount} images removed`
                );
            }

            if (updatedCount > 0) {
                notification.info(
                    'Library Sync',
                    updatedCount === 1 ? '1 image updated' : `${updatedCount} images updated`
                );
            }
        }).then(unlisten => {
            unlistenBatchChange = unlisten;
        });

        onCleanup(() => {
            if (unlistenBatchChange) unlistenBatchChange();
        });
    });
};
