import { onCleanup, createEffect } from "solid-js";
import { ImageItem } from "../../types";
import { dndRegistry, setDragItem, currentDragItem, setDropTargetId, currentDropTargetId } from "./dnd-core";
import { createDragGhost } from "./ghost";

export interface AssetDnDParams {
    item: ImageItem;
    selected: boolean;
    selectedIds: (number | string)[];
    allItems: ImageItem[];
}

/**
 * SolidJS Directive for Asset Drag and Drop.
 * Encapsulates complex drag ghost creation, selection-aware dragging,
 * and drop target logic for tags and images.
 */
export function assetDnD(el: HTMLElement, accessor: () => AssetDnDParams) {
    // Local reactive state for this specific element
    let dragCounter = 0;
    
    // Reset internal state if the element is recycled for a different item (virtualization)
    createEffect(() => {
        accessor().item.id;
        dragCounter = 0;
    });

    // Sync class with GLOBAL reactive target state
    // This solves the virtualization bug where reused DOM nodes would keep old highlights
    createEffect(() => {
        const targetId = currentDropTargetId();
        const params = accessor();
        const myId = params.item.id;
        
        if (targetId !== null && targetId === myId) {
            el.classList.add("drop-target-active");
        } else {
            el.classList.remove("drop-target-active");
        }
    });

    // Auto-cleanup when global dragging ends anywhere
    createEffect(() => {
        if (!currentDragItem()) {
            dragCounter = 0;
            // No need to manually clear classes here as the effect above handles it 
            // once currentDropTargetId is reset globally
        }
    });

    const handleDragStart = (e: DragEvent) => {
        e.stopPropagation();
        if (!e.dataTransfer) return;

        const { item, selected, selectedIds, allItems } = accessor();
        
        let ids = [item.id];
        if (selected && selectedIds.includes(item.id)) {
            ids = [...selectedIds] as number[];
        }

        const data = { type: "IMAGE", payload: { ids } };
        setDragItem(data as any);

        e.dataTransfer.effectAllowed = "copyMove";
        e.dataTransfer.setData("application/json", JSON.stringify(data));

        const draggedItems: ImageItem[] = [];
        const validPaths: string[] = [];
        const cleanPaths: string[] = [];

        ids.forEach(id => {
            const found = allItems.find(i => i.id === id);
            if (found) {
                draggedItems.push(found);
                if (found.path) {
                    validPaths.push(`file://${found.path}`);
                    cleanPaths.push(found.path);
                }
            }
        });

        if (validPaths.length > 0) {
            e.dataTransfer.setData("text/uri-list", validPaths.join("\r\n"));
            e.dataTransfer.setData("text/plain", cleanPaths.join("\n"));
        }

        const ghost = createDragGhost(draggedItems);
        e.dataTransfer.setDragImage(ghost, 0, 0);

        setTimeout(() => {
            if (ghost.parentNode) {
                document.body.removeChild(ghost);
            }
        }, 0);
    };

    const handleDragEnd = () => {
        setDragItem(null);
        setDropTargetId(null);
        dragCounter = 0;
    };

    const handleDragEnter = (e: DragEvent) => {
        e.preventDefault();
        dragCounter++;
        
        if (dragCounter === 1) {
            const strategy = dndRegistry.get("IMAGE");
            const dragging = currentDragItem();
            if (strategy && strategy.onDragOver && dragging && strategy.onDragOver(dragging)) {
                setDropTargetId(accessor().item.id);
            }
        }
    };

    const handleDragOver = (e: DragEvent) => {
        e.preventDefault();
        const dragging = currentDragItem();
        if (!dragging) return;

        const strategy = dndRegistry.get("IMAGE");
        if (strategy && strategy.onDragOver && strategy.onDragOver(dragging)) {
            e.dataTransfer!.dropEffect = "copy";
            // Ensure target is set if enter was missed or ID changed due to virtualization
            const myId = accessor().item.id;
            if (dragCounter > 0 && currentDropTargetId() !== myId) {
                setDropTargetId(myId);
            }
        }
    };

    const handleDragLeave = () => {
        dragCounter = Math.max(0, dragCounter - 1);
        if (dragCounter === 0) {
            if (currentDropTargetId() === accessor().item.id) {
                setDropTargetId(null);
            }
        }
    };

    const handleDrop = async (e: DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        dragCounter = 0;
        setDropTargetId(null);

        try {
            const json = e.dataTransfer?.getData("application/json");
            if (json) {
                const data = JSON.parse(json);
                const strategy = dndRegistry.get("IMAGE");
                if (strategy && strategy.accepts(data)) {
                    const { item } = accessor();
                    await strategy.onDrop(data, item.id);
                }
            }
        } catch (err) {
            console.error("Drop failed", err);
        } finally {
            setDragItem(null);
            setDropTargetId(null);
        }
    };

    // Attach listeners
    el.setAttribute("draggable", "true");
    el.addEventListener("dragstart", handleDragStart);
    el.addEventListener("dragend", handleDragEnd);
    el.addEventListener("dragenter", handleDragEnter);
    el.addEventListener("dragover", handleDragOver);
    el.addEventListener("dragleave", handleDragLeave);
    el.addEventListener("drop", handleDrop);

    onCleanup(() => {
        el.removeEventListener("dragstart", handleDragStart);
        el.removeEventListener("dragend", handleDragEnd);
        el.removeEventListener("dragenter", handleDragEnter);
        el.removeEventListener("dragover", handleDragOver);
        el.removeEventListener("dragleave", handleDragLeave);
        el.removeEventListener("drop", handleDrop);
    });
}

// Add TypeScript support for the directive
declare module "solid-js" {
    namespace JSX {
        interface Directives {
            assetDnD: AssetDnDParams;
        }
    }
}
