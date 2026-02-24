import { createShortcut, createConditionalScope, SCOPE_PRIORITIES } from '../../../../core/input';
import { TreeNode } from '../types';

interface UseTreeNavigationOptions {
    /** Accessor for the node being navigated */
    node: () => TreeNode<unknown>;
    /** Whether the node is currently in edit mode */
    isEditing: () => boolean;
    /** Whether the node has children */
    hasChildren: () => boolean;
    /** Current expansion state */
    isExpanded: () => boolean;
    /** Callback to select the node */
    onSelect?: () => void;
    /** Callback to toggle expansion */
    onToggle?: (expanded: boolean) => void;
    /** Callback to cancel editing */
    onEditCancel?: () => void;
    /** Component focus state */
    isFocused: () => boolean;
}

/**
 * Hook for managing keyboard navigation and focus scopes for a tree item.
 * Aligns with the global input system.
 */
export const useTreeNavigation = (options: UseTreeNavigationOptions) => {
    // Automatically manage 'editing' scope based on editing state and focus
    // This allows global shortcuts to be shadowed when typing in the tree
    createConditionalScope(
        'editing',
        () => options.isEditing() && options.isFocused(),
        SCOPE_PRIORITIES.editing,
        true
    );

    // Keyboard navigation when NOT editing
    createShortcut({
        keys: ['Enter'],
        action: () => {
            if (!options.isEditing()) {
                options.onSelect?.();
            }
        },
        enabled: () => !options.isEditing() && options.isFocused()
    });

    createShortcut({
        keys: ['ArrowRight'],
        action: () => {
            if (options.hasChildren() && !options.isExpanded()) {
                options.onToggle?.(true);
            }
        },
        enabled: () => !options.isEditing() && options.isFocused(),
        preventDefault: true
    });

    createShortcut({
        keys: ['ArrowLeft'],
        action: () => {
            if (options.hasChildren() && options.isExpanded()) {
                options.onToggle?.(false);
            }
        },
        enabled: () => !options.isEditing() && options.isFocused(),
        preventDefault: true
    });

    // Edit mode specific shortcuts
    createShortcut({
        keys: ['Escape'],
        action: () => {
            if (options.isEditing()) {
                options.onEditCancel?.();
            }
        },
        enabled: () => options.isEditing() && options.isFocused()
    });

    return {};
};
