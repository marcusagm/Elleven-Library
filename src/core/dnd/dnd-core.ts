// Define types for Draggable Items
export interface DragItem {
    type: "IMAGE" | "TAG";
    payload: any;
}

// Strategy Interface
export interface DropStrategy {
    accepts(item: DragItem): boolean;
    onDrop(item: DragItem, targetId: number | string): Promise<void>;
    onDragOver?(item: DragItem): boolean; // valid drop target?
}

// Registry to hold strategies
class DndStrategyRegistry {
    private strategies: Map<string, DropStrategy> = new Map();

    register(targetType: string, strategy: DropStrategy) {
        console.log("dnd-core:register", targetType, strategy);
        this.strategies.set(targetType, strategy);
    }

    get(targetType: string): DropStrategy | undefined {
        console.log("dnd-core: get", targetType);
        return this.strategies.get(targetType);
    }
}

export const dndRegistry = new DndStrategyRegistry();

// Global Drag State (for DragOver checks)
export let currentDragItem: DragItem | null = null;

export const setDragItem = (item: DragItem | null) => {
    console.log("dnd-core:setDragItem", item);
    currentDragItem = item;
};
