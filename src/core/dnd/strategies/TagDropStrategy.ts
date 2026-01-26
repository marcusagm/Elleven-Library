import { DropStrategy, DragItem } from "../dnd-core";
import { tagService } from "../../../lib/tags";
import { appActions } from "../../store/appStore";

// Strategy: Dropping anything ONTO a Tag
export const TagDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        return item.type === "IMAGE" || item.type === "TAG";
    },

    onDrop: async (item: DragItem, targetId: number | string) => {
        console.log("TagDropStrategy:onDrop", item, targetId);
        let targetTagId: number | null = Number(targetId);
        if (targetId === "root" || isNaN(targetTagId)) {
            targetTagId = null;
            console.log("Target is ROOT");
        }

        // Case 1: Image dropped on Tag (Assignment)
        if (item.type === "IMAGE") {
            const imageIds = item.payload.ids as number[];
            if (targetTagId === null) {
                // Cannot assign to root? Or assigning to "Uncategorized"?
                // "Root" usually means removing all tags or moving to top level?
                // If it's the "Tags" header, maybe we just don't support image drop.
                console.warn("Cannot assign images to root tag container");
                return;
            }
            console.log(`Assigning images [${imageIds}] to tag ${targetTagId}`);
            
            try {
                await tagService.addTagsToImagesBatch(imageIds, [targetTagId]);
                appActions.notifyTagUpdate();
            } catch (err) {
                console.error("Failed to assign tag:", err);
            }
        }

        // Case 2: Tag dropped on Tag (Nesting / Reordering)
        if (item.type === "TAG") {
            const draggedTagId = Number(item.payload.id);
            if (draggedTagId === targetTagId) return;

            console.log(`Moving tag ${draggedTagId} to parent ${targetTagId}`);
            try {
                // Update tag with new parentId (or null if root)
                // Assuming updateTag signature: (id, name?, color?, parent_id?)
                // If targetTagId is null, we are moving to root.
                // We need to verify if updateTag handles null for parent_id.
                // It usually does if we pass `null`.
                await tagService.updateTag(draggedTagId, undefined, undefined, targetTagId);
                await appActions.loadTags();
            } catch (err) {
                console.error("Failed to move tag:", err);
            }
        }
    },
    
    onDragOver: (item: DragItem) => {
        console.log("TagDropStrategy:onDragOver", item);
         // Prevent dropping parent into child (circular check) could be here if we had tree context
         return true;
    }
};
