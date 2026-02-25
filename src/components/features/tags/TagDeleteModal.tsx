import { ConfirmModal } from '../../ui';
import { Component, Show } from 'solid-js';
import { TreeNode } from '../../ui/TreeView';
import { tagService } from '../../../lib/tags';
import { useMetadata, useNotification } from '../../../core/hooks';
import './tag-delete-modal.css';

/**
 * Properties for the TagDeleteModal component.
 */
interface TagDeleteModalProperties {
    /** Whether the modal is open. */
    isOpen: boolean;
    /** Callback to close the modal. */
    onClose: () => void;
    /** The tag node to be deleted. */
    node: TreeNode | null;
}

/**
 * Modal to confirm the deletion of a tag and its descendants.
 *
 * @param componentProperties - Properties for the component.
 * @returns The rendered TagDeleteModal.
 */
export const TagDeleteModal: Component<TagDeleteModalProperties> = componentProperties => {
    const { loadTags } = useMetadata();
    const notification = useNotification();

    /**
     * Recursively retrieves all descendant identifiers of a given node.
     *
     * @param node - The starting tree node.
     * @returns An array of descendant numeric identifiers.
     */
    const getAllDescendantIdentifiers = (node: TreeNode): number[] => {
        let identifiers: number[] = [];
        if (node.children) {
            node.children.forEach(child => {
                identifiers.push(Number(child.id));
                identifiers = [...identifiers, ...getAllDescendantIdentifiers(child)];
            });
        }
        return identifiers;
    };

    /**
     * Handles the confirmation of tag deletion.
     */
    const handleConfirm = async () => {
        const node = componentProperties.node;
        if (!node) {
            return;
        }

        const tagName = node.label;
        const parentIdentifier = (node.data as Record<string, unknown>)?.parent_id as number;
        const tagColor = (node.data as Record<string, unknown>)?.color as string;

        try {
            const descendantIdentifiers = getAllDescendantIdentifiers(node);
            for (const childIdentifier of descendantIdentifiers) {
                await tagService.deleteTag(childIdentifier);
            }
            await tagService.deleteTag(Number(node.id));
            await loadTags();

            notification.success('Tag Deleted', `Removed "${tagName}"`, {
                label: 'Undo',
                onClick: async () => {
                    try {
                        await tagService.createTag(tagName, parentIdentifier, tagColor);
                        await loadTags();
                        notification.success('Restored', `Tag "${tagName}" restored`);
                    } catch {
                        notification.error('Failed to restore tag');
                    }
                }
            });
        } catch (error) {
            console.error('Delete failed:', error);
            notification.error('Failed to Delete Tag');
        } finally {
            componentProperties.onClose();
        }
    };

    /**
     * Calculates the number of descendant tags.
     */
    const descendantCount = () =>
        componentProperties.node ? getAllDescendantIdentifiers(componentProperties.node).length : 0;

    return (
        <ConfirmModal
            isOpen={componentProperties.isOpen}
            onClose={componentProperties.onClose}
            onConfirm={handleConfirm}
            title="Delete Tag"
            kind="danger"
            confirmText="Delete"
            message=""
        >
            <div class="tag-delete-modal-content">
                <p>
                    Are you sure you want to delete tag{' '}
                    <strong>"{componentProperties.node?.label}"</strong>?
                </p>
                <Show when={descendantCount() > 0}>
                    <p class="tag-delete-warning">
                        This will also delete <strong>{descendantCount()}</strong> child tags. This
                        action cannot be undone.
                    </p>
                </Show>
            </div>
        </ConfirmModal>
    );
};
