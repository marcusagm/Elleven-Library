import { DropStrategy, DragItem } from '../dnd-core';
import { tagService, type Tag } from '../../../lib/tags';
import { metadataActions, metadataState } from '../../store/metadataStore';
import { toast } from '../../../components/ui';

/** Handle dropping images onto a tag (batch assignment) */
async function handleImageDrop(imageIds: number[], targetTagId: number | null): Promise<void> {
    if (targetTagId === null) {
        console.warn('Cannot assign images to root tag container');
        return;
    }
    try {
        await tagService.addTagsToImagesBatch(imageIds, [targetTagId]);
        metadataActions.notifyTagUpdate();

        const tagName = metadataState.tags.find(tag => tag.id === targetTagId)?.name || 'Tag';
        toast.success('Tag Applied', {
            description: `Added "${tagName}" to ${imageIds.length} item(s)`
        });
    } catch (error) {
        console.error('Failed to assign tag:', error);
        toast.error('Failed to Apply Tag');
    }
}

/** Determine the new parent ID based on drop position */
function resolveNewParentId(
    targetTagId: number | null,
    position: 'before' | 'inside' | 'after'
): number | null {
    if (position === 'inside') return targetTagId;
    if (targetTagId === null) return null;
    const targetTag = metadataState.tags.find(tag => tag.id === targetTagId);
    return targetTag ? targetTag.parent_id : null;
}

/** Build the ordered sibling list with the dragged tag inserted at the correct position */
function buildReorderedSiblings(
    allTags: Tag[],
    draggedTagId: number,
    newParentId: number | null,
    targetTagId: number | null,
    position: 'before' | 'inside' | 'after'
): Tag[] | null {
    const siblings = allTags
        .filter(tag => tag.parent_id === newParentId && tag.id !== draggedTagId)
        .sort(
            (tagA, tagB) =>
                tagA.order_index - tagB.order_index || tagA.name.localeCompare(tagB.name)
        );

    let insertIndex = siblings.length;

    if (position !== 'inside') {
        const targetIndex = siblings.findIndex(tag => tag.id === targetTagId);
        if (targetIndex !== -1) {
            insertIndex = position === 'before' ? targetIndex : targetIndex + 1;
        }
    }

    const draggedTag = allTags.find(tag => tag.id === draggedTagId);
    if (!draggedTag) return null;

    siblings.splice(insertIndex, 0, draggedTag);
    return siblings;
}

/** Emit update calls for reordered siblings */
function createReorderUpdates(
    siblings: Tag[],
    draggedTagId: number,
    newParentId: number | null
): Promise<void>[] {
    return siblings.map((tag, index) => {
        const newOrder = index * 100;
        const isDragged = tag.id === draggedTagId;

        if (isDragged) {
            return tagService.updateTag(
                tag.id,
                undefined,
                undefined,
                newParentId === null ? 0 : newParentId,
                newOrder
            );
        } else if (tag.order_index !== newOrder) {
            return tagService.updateTag(
                tag.id,
                undefined,
                undefined,
                tag.parent_id === null ? 0 : tag.parent_id,
                newOrder
            );
        }
        return Promise.resolve();
    });
}

/** Handle dropping a tag to reorder or nest it */
async function handleTagDrop(
    draggedTagId: number,
    targetTagId: number | null,
    position: 'before' | 'inside' | 'after'
): Promise<void> {
    try {
        const newParentId = resolveNewParentId(targetTagId, position);
        const siblings = buildReorderedSiblings(
            metadataState.tags,
            draggedTagId,
            newParentId,
            targetTagId,
            position
        );
        if (!siblings) return;

        const updates = createReorderUpdates(siblings, draggedTagId, newParentId);
        await Promise.all(updates);
        metadataActions.loadTags();
    } catch (error) {
        console.error('Failed to move tag:', error);
    }
}

// Strategy: Dropping anything ONTO a Tag
export const TagDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        return item.type === 'IMAGE' || item.type === 'TAG';
    },

    onDrop: async (
        item: DragItem,
        targetId: number | string,
        position: 'before' | 'inside' | 'after' = 'inside'
    ) => {
        let targetTagId: number | null = Number(targetId);

        if (targetId === 'root' || isNaN(targetTagId)) {
            targetTagId = null;
        }

        if (item.type === 'IMAGE') {
            const imagePayload = item.payload as Record<string, unknown>;
            const imageIds = imagePayload.ids as number[];
            await handleImageDrop(imageIds, targetTagId);
        }

        if (item.type === 'TAG') {
            const draggedTagId = Number(item.payload.id);
            await handleTagDrop(draggedTagId, targetTagId, position);
        }
    },

    onDragOver: () => {
        return true;
    }
};
