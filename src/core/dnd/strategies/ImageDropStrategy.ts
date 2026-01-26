import { DropStrategy, DragItem } from "../dnd-core";
import { tagService } from "../../../lib/tags";
import { appActions } from "../../store/appStore";

// Strategy: Dropping anything ONTO an Image
export const ImageDropStrategy: DropStrategy = {
    accepts: (item: DragItem) => {
        // Only accept TAGS being dropped on images
        return item.type === "TAG";
    },

    onDrop: async (item: DragItem, targetId: number | string) => {
        console.log("ImageDropStrategy:onDrop", item, targetId);
        const targetImageId = Number(targetId);
        
        if (item.type === "TAG") {
            const tagId = Number(item.payload.id);
            console.log(`Assigning tag ${tagId} to image ${targetImageId}`);
            
            try {
                // If multiple images are selected, user expects to apply to ALL selected?
                // Logic: If the target image is part of selection, apply to all.
                // If target image is NOT in selection, apply only to target.
                // BUT: We don't have easy access to Store state here without importing store directly.
                // We'll trust the payload or just apply to target for simplicity now, 
                // typically we drag ONTO an image means that image gets the tag.
                
                await tagService.addTagsToImagesBatch([targetImageId], [tagId]);
                appActions.notifyTagUpdate();
            } catch (err) {
                console.error("Failed to assign tag to image:", err);
            }
        }
    },
    
    onDragOver: (item: DragItem) => {
        console.log("ImageDropStrategy:onDragOver", item);
        return item.type === "TAG";
    }
};
