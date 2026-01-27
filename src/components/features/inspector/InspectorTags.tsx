import { Component, createMemo, createResource } from "solid-js";
import { Tag as TagIcon } from "lucide-solid";
import { useAppStore, appActions } from "../../../core/store/appStore";
import { tagService } from "../../../lib/tags";
import { TagInput, TagOption } from "../../ui/TagInput";
import { AccordionItem } from "../../ui/Accordion";
import "./inspector.css";

export const InspectorTags: Component = () => {
    const { state } = useAppStore();

    // Active item logic (single or multi?)
    // For now, let's assume we handle the active selection logic inside here or pass it as props?
    // The plan says "Orchestration". So maybe this component should receive the selection?
    // But `FileInspector` has the `activeItem`.
    // Multi-selection logic is more complex.
    
    // Let's rely on the store's selection for now to determine "active".
    // If multiple items, we might need to show common tags or mixed state.
    // For simple start, let's handle the single active item as per current FileInspector logic, 
    // or upgrade to multi-selection handling if we are ready.
    // The previous implementation used `state.selection`.
    
    const activeId = createMemo(() => {
        if (state.selection.length === 0) return null;
        return state.selection[state.selection.length - 1]; // Last selected
    });

    const [itemTags, { refetch: refetchTags }] = createResource(
        () => ({ id: activeId(), trigger: useAppStore().tagUpdateTrigger() }),
        async ({ id }) => {
            if (!id) return [];
            return await tagService.getTagsForImage(id);
        }
    );

    const tagValue = createMemo<TagOption[]>(() => {
        return (itemTags() || []).map(t => ({
            id: t.id,
            label: t.name,
            color: t.color || undefined
        }));
    });

    const allTagsOptions = createMemo<TagOption[]>(() => {
        return state.tags.map(t => ({
            id: t.id,
            label: t.name,
            color: t.color || undefined
        }));
    });

    const handleTagsChange = async (newTags: TagOption[]) => {
        const current = itemTags() || [];
        const currentIds = new Set(current.map(t => t.id));
        const newIds = new Set(newTags.map(t => Number(t.id)));
        
        const selection = state.selection;
        if (selection.length === 0) return;

        // Added items
        const added = newTags.filter(t => !currentIds.has(Number(t.id)));
        if (added.length > 0) {
            await tagService.addTagsToImagesBatch([...selection], added.map(t => Number(t.id)));
        }

        // Removed items
        const removed = current.filter(t => !newIds.has(t.id));
        for (const t of removed) {
            for (const itemId of selection) {
               await tagService.removeTagFromImage(itemId, t.id);
            }
        }
        
        appActions.notifyTagUpdate();
        refetchTags();
    };

    const handleCreateTag = async (name: string) => {
        const selection = state.selection;
        if (selection.length === 0) return;
        
        const newTagId = await tagService.createTag(name);
        await appActions.loadTags(); 
        
        await tagService.addTagsToImagesBatch([...selection], [newTagId]);
        appActions.notifyTagUpdate();
        refetchTags();
    };

    return (
        <AccordionItem 
            value="tags" 
            title="Tags" 
            defaultOpen 
            icon={<TagIcon size={14} />}
        >
            <div class="inspector-tags-wrapper">
                <TagInput 
                    value={tagValue()} 
                    onChange={handleTagsChange} 
                    suggestions={allTagsOptions()}
                    onCreate={handleCreateTag}
                    placeholder="Add tags..."
                />
            </div>
        </AccordionItem>
    );
};
