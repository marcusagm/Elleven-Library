import { DropStrategy, DragItem, DndActionResult } from '../dnd-core';
import { ErrorCode } from '../../types/actions';

/**
 * Strategy: Dropping items onto a Tag target.
 */
export const TagDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        return item.type === 'ASSET' || item.type === 'TAG';
    },

    onDrop: async (
        item: DragItem,
        targetId: number | string,
        position: 'before' | 'inside' | 'after' = 'inside'
    ): Promise<DndActionResult> => {
        let targetTagId: string | null = String(targetId);
        if (targetId === 'root' || targetId === undefined || targetId === null) {
            targetTagId = null;
        }

        if (item.type === 'ASSET') {
            const { libraryActions } = await import('../../store/library');
            const { selectionState } = await import('../../store/selectionStore');

            let assetIds: string[] = [String(item.payload.id)];
            if (selectionState.selectedIds.includes(String(item.payload.id))) {
                assetIds = [...selectionState.selectedIds];
            }

            if (targetTagId !== null) {
                return await libraryActions.applyTagToAssets(assetIds, targetTagId);
            }
        }

        if (item.type === 'TAG') {
            const { metadataActions } = await import('../../store/metadata');
            const draggedTagId = String(item.payload.id);
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
