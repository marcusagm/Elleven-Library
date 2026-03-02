import { createSignal } from 'solid-js';
import { setDropTargetId, currentDropTargetId, currentDragItem, DragItem } from '../dnd';
import { useDndHandlers } from './useDndHandlers';

/**
 * Specialized hook for managing drop zone behavior on assets (AssetItems).
 * Handles drag counters to prevent flickering and coordinates with the DND handler.
 *
 * @param getAssetId - A reactive accessor for the asset ID. This ensures the hook
 * stays correctly bound to the current item even if the underlying DOM node is
 * recycled in a virtualized list.
 */
export const useAssetDropZone = (getAssetId: () => number) => {
    const [dragCounter, setDragCounter] = createSignal(0);
    const { handleDrop } = useDndHandlers();

    /**
     * Helper to get the current ID from the reactive accessor.
     */
    const getTargetId = () => getAssetId();

    const isDropTarget = () => currentDropTargetId() === getTargetId();

    const onDragEnter = (event: DragEvent) => {
        event.preventDefault();
        const nextCounter = dragCounter() + 1;
        setDragCounter(nextCounter);

        if (nextCounter === 1) {
            const dragging = currentDragItem();
            // Assets only accept TAG drops currently for assignment
            if (dragging?.type === 'TAG') {
                setDropTargetId(getTargetId());
            }
        }
    };

    const onDragOver = (event: DragEvent) => {
        event.preventDefault();
        const dragging = currentDragItem();
        if (dragging?.type === 'TAG') {
            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'copy';
            }
            // Fallback for missed enter events due to rapid movement or virtualization
            const currentId = getTargetId();
            if (dragCounter() > 0 && currentDropTargetId() !== currentId) {
                setDropTargetId(currentId);
            }
        }
    };

    const onDragLeave = () => {
        const nextCounter = Math.max(0, dragCounter() - 1);
        setDragCounter(nextCounter);
        if (nextCounter === 0 && currentDropTargetId() === getTargetId()) {
            setDropTargetId(null);
        }
    };

    const onDrop = async (event: DragEvent) => {
        event.preventDefault();
        event.stopPropagation();
        setDragCounter(0);
        setDropTargetId(null);

        try {
            const rawJsonData = event.dataTransfer?.getData('application/json');
            if (rawJsonData) {
                const droppedItem: DragItem = JSON.parse(rawJsonData);
                // Asset cards specifically only allow TAG drops for tag assignment
                if (droppedItem.type === 'TAG') {
                    await handleDrop(droppedItem, getTargetId(), 'ASSET');
                }
            }
        } catch (error) {
            console.error('Asset drop failed:', error);
        }
    };

    return {
        isDropTarget,
        dragHandlers: {
            onDragEnter,
            onDragOver,
            onDragLeave,
            onDrop
        }
    };
};
