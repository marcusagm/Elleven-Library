import { useNotification } from './useNotification';
import { dndRegistry, DragItem, setDragItem, DndActionResult, DndSuccessData } from '../dnd';
import { ErrorCode } from '../types/actions';

/**
 * Hook to handle drag and drop operations with associated UI feedback (toasts).
 * Centralizes the logic for executing drops through strategies and notifying the user.
 */
export const useDndHandlers = () => {
    const notification = useNotification();

    /**
     * Internal helper to display appropriate success notifications based on drop results.
     */
    const notifySuccess = (item: DragItem, data: DndSuccessData | void, position: string) => {
        // If data contains a tagName, it means a tag was applied/assigned (Asset <-> Tag)
        if (data && typeof data === 'object' && 'tagName' in data) {
            notification.success(
                'Tag Applied',
                `Added "${data.tagName}" to ${data.count} image(s)`
            );
            return;
        }

        // If it's a TAG item, it's a hierarchy management operation (nesting or reordering)
        if (item.type === 'TAG') {
            if (position === 'inside') {
                notification.success('Tag Moved', 'Tag nested successfully');
            } else {
                notification.success('Tag Reordered', 'Position updated');
            }
        }
    };

    /**
     * Executes a drop operation for a given item and target.
     * Finds the appropriate strategy, executes it, and shows a success/error notification.
     */
    const handleDrop = async (
        item: DragItem,
        targetId: number | string,
        targetType: 'IMAGE' | 'TAG',
        position: 'before' | 'inside' | 'after' = 'inside'
    ): Promise<DndActionResult> => {
        try {
            const strategy = dndRegistry.get(targetType);
            if (!strategy || !strategy.accepts(item)) {
                return {
                    success: false,
                    error: { code: ErrorCode.VALIDATION_ERROR, message: 'No valid strategy found' }
                };
            }

            const result = await strategy.onDrop(item, targetId, position);

            if (result.success) {
                notifySuccess(item, result.data, position);
            } else if (result.error) {
                notification.error(result.error.message || 'Drop failed');
            }

            return result;
        } catch (error) {
            console.error('Dnd Execution Error:', error);
            notification.error('An unexpected error occurred during drop');
            return {
                success: false,
                error: { code: ErrorCode.INTERNAL_ERROR, message: 'Unexpected DND error' }
            };
        } finally {
            setDragItem(null);
        }
    };

    return { handleDrop };
};
