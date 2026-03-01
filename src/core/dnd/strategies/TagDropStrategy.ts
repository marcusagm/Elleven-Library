import { DropStrategy, DragItem, DndActionResult } from '../dnd-core';
import { ErrorCode } from '../../types/actions';

/**
 * Strategy: Dropping items onto a Tag target.
 */
export const TagDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        return item.type === 'IMAGE' || item.type === 'TAG';
    },

    onDrop: async (
        item: DragItem,
        targetId: number | string,
        position: 'before' | 'inside' | 'after' = 'inside'
    ): Promise<DndActionResult> => {
        let targetTagId: number | null = Number(targetId);
        if (targetId === 'root' || isNaN(targetTagId)) {
            targetTagId = null;
        }

        if (item.type === 'IMAGE') {
            const { libraryActions } = await import('../../store/library');
            const { selectionState } = await import('../../store/selectionStore');

            let imageIds = [item.payload.id];
            if (selectionState.selectedIds.includes(item.payload.id)) {
                imageIds = [...selectionState.selectedIds];
            }

            if (targetTagId !== null) {
                return await libraryActions.applyTagToImages(imageIds, targetTagId);
            }
        }

        if (item.type === 'TAG') {
            const { metadataActions } = await import('../../store/metadata');
            const draggedTagId = Number(item.payload.id);
            return await metadataActions.moveTag(draggedTagId, targetTagId, position);
        }

        return {
            success: false,
            error: { code: ErrorCode.VALIDATION_ERROR, message: 'Invalid drop operation' }
        };
    },

    onDragOver: () => {
        return true;
    }
};
