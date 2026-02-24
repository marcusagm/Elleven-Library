// Define types for Draggable Items
export interface TagDragPayload {
    id: number;
    name?: string;
}

export interface DragItem {
    type: 'IMAGE' | 'TAG';
    payload: TagDragPayload | Record<string, unknown>;
}

// Strategy Interface
export interface DropStrategy {
    /** Determines if this strategy can handle the given dragged item. */
    accepts(item: DragItem): boolean;
    /** Performs the actual drop operation logic. */
    onDrop(
        item: DragItem,
        targetId: number | string,
        position?: 'before' | 'inside' | 'after'
    ): Promise<void>;
    /** Optional hook to determine if the current drag position is a valid drop target. */
    onDragOver?(item: DragItem): boolean;
}

// Registry to hold strategies
class DndStrategyRegistry {
    private strategies: Map<string, DropStrategy> = new Map();

    register(targetType: string, strategy: DropStrategy) {
        this.strategies.set(targetType, strategy);
    }

    get(targetType: string): DropStrategy | undefined {
        return this.strategies.get(targetType);
    }
}

import { createSignal } from 'solid-js';

export const dndRegistry = new DndStrategyRegistry();

// Global Drag State (Signal for Reactivity)
export const [currentDragItem, setDragItem] = createSignal<DragItem | null>(null);
export const [currentDropTargetId, setDropTargetId] = createSignal<number | string | null>(null);
