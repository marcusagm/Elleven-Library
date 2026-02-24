import { createSignal, createMemo } from 'solid-js';
import { currentDragItem, setDragItem, dndRegistry, DragItem } from '../../../../core/dnd';
import { TreeNode, TreeDropPosition } from '../types';

interface UseTreeDragDropOptions {
    /** Accessor for the node being focused for drag/drop */
    node: () => TreeNode<unknown>;
    /** Whether drag and drop is enabled */
    isEnabled: () => boolean;
    /** Whether the node is currently in edit mode (disables drag) */
    isEditing: () => boolean;
    /** Accessor for the drag type identifier of this tree */
    dragType: () => string | undefined;
    /** Accessor for the list of drag types this tree accepts */
    acceptedDragTypes: () => string[] | undefined;
    /** Function to perform custom validation for drop operations */
    isValidDrop: (dragged: DragItem, target: TreeNode<unknown>) => boolean;
}

/**
 * Hook for managing generic drag-and-drop state and operations for a tree item.
 * Fully decoupled from specific domain types.
 */
export const useTreeDragDrop = (options: UseTreeDragDropOptions) => {
    const [dropPosition, setDropPosition] = createSignal<TreeDropPosition | null>(null);

    /**
     * Validation memo to track if current drag-over is valid.
     */
    const validationStatus = createMemo(() => {
        const draggingItem = currentDragItem();
        if (!draggingItem) {
            return { isValid: true };
        }

        // 1. Type validation
        const acceptedTypes = options.acceptedDragTypes();
        const isTypeAccepted = acceptedTypes ? acceptedTypes.includes(draggingItem.type) : true;

        if (!isTypeAccepted) {
            return { isValid: false };
        }

        const targetNode = options.node();
        const selfDragType = options.dragType();

        // 2. Self-drop prevention
        const isSelfDrop =
            draggingItem.type === selfDragType &&
            String(draggingItem.payload.id) === String(targetNode.id);

        if (isSelfDrop) {
            return { isValid: false };
        }

        // 3. Custom external validation (Domain rules)
        if (!options.isValidDrop(draggingItem, targetNode)) {
            return { isValid: false };
        }

        return { isValid: true };
    });

    const isDraggingSource = () => {
        const item = currentDragItem();
        const selfDragType = options.dragType();
        return (
            item?.type === selfDragType && String(item?.payload?.id) === String(options.node().id)
        );
    };

    const isDropInvalid = () => {
        const item = currentDragItem();
        return item && !validationStatus().isValid;
    };

    const handleDragStart = (event: DragEvent) => {
        const selfDragType = options.dragType();
        if (!options.isEnabled() || options.isEditing() || !selfDragType) {
            return;
        }

        event.stopPropagation();
        if (event.dataTransfer) {
            const dragData: DragItem = {
                // We trust the provided dragType matches our system's DragItem identifiers
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                type: selfDragType as any,
                payload: { id: options.node().id }
            };

            setDragItem(dragData);
            event.dataTransfer.effectAllowed = 'move';
            event.dataTransfer.setData('application/json', JSON.stringify(dragData));
        }
    };

    const handleDragEnd = () => {
        setDragItem(null);
        setDropPosition(null);
    };

    const handleDragOver = (event: DragEvent) => {
        event.preventDefault();

        if (!options.isEnabled() || !validationStatus().isValid) {
            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'none';
            }
            setDropPosition(null);
            return;
        }

        const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
        const mouseYPosition = event.clientY - rect.top;
        const totalElementHeight = rect.height;
        const edgeThresholdDistance = totalElementHeight * 0.25;

        let position: TreeDropPosition = 'inside';

        const draggingItem = currentDragItem();
        const selfDragType = options.dragType();
        const isSiblingDragMatch = draggingItem?.type === selfDragType;

        // Position indicators ('before'/'after') only for sibling moves
        if (isSiblingDragMatch) {
            if (mouseYPosition < edgeThresholdDistance) {
                position = 'before';
            } else if (mouseYPosition > totalElementHeight - edgeThresholdDistance) {
                position = 'after';
            }
        }

        setDropPosition(position);
        if (event.dataTransfer) {
            event.dataTransfer.dropEffect = isSiblingDragMatch ? 'move' : 'copy';
        }
    };

    const handleDragLeave = () => {
        setDropPosition(null);
    };

    const handleDrop = async (event: DragEvent) => {
        event.preventDefault();
        event.stopPropagation();

        const finalDropPosition = dropPosition();
        setDropPosition(null);

        if (!validationStatus().isValid) {
            return;
        }

        try {
            const rawJson = event.dataTransfer?.getData('application/json');
            if (rawJson) {
                const droppedItem: DragItem = JSON.parse(rawJson);
                const dropStrategy = dndRegistry.get(droppedItem.type);

                if (dropStrategy && dropStrategy.accepts(droppedItem)) {
                    // TreeView component acts as a generic bridge to the DND strategy system
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    await (dropStrategy as any).onDrop(
                        droppedItem,
                        options.node().id,
                        finalDropPosition || 'inside'
                    );
                }
            }
        } catch (error) {
            console.error('Hierarchical drop operation failed:', error);
        } finally {
            setDragItem(null);
        }
    };

    return {
        dropPosition,
        isDraggingSource,
        isDropInvalid,
        handleDragStart,
        handleDragEnd,
        handleDragOver,
        handleDragLeave,
        handleDrop
    };
};
