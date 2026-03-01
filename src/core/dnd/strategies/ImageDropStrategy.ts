import { DropStrategy, DragItem, DndActionResult } from '../dnd-core';
import { ErrorCode } from '../../types/actions';

/**
 * Strategy: Dropping items onto an Image target.
 */
export const ImageDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        // Only accept TAGS being dropped on images
        return item.type === 'TAG';
    },

    onDrop: async (item: DragItem, targetId: number | string): Promise<DndActionResult> => {
        if (item.type === 'TAG') {
            const { libraryActions } = await import('../../store/library');
            const targetImageId = Number(targetId);
            const tagId = Number(item.payload.id);

            // Emit intention to library store
            return await libraryActions.applyTagToTarget(tagId, targetImageId);
        }
        return {
            success: false,
            error: { code: ErrorCode.VALIDATION_ERROR, message: 'Invalid drag item type' }
        };
    },

    onDragOver: (item: DragItem) => {
        return item.type === 'TAG';
    }
};
