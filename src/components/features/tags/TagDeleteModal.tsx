import { Component, Show } from "solid-js";
import "./tag-delete-modal.css";
import { ConfirmModal } from "../../ui/Modal";
import { TreeNode } from "../../ui/TreeView";
import { tagService } from "../../../lib/tags";
import { useMetadata } from "../../../core/hooks";

interface TagDeleteModalProps {
    isOpen: boolean;
    onClose: () => void;
    node: TreeNode | null;
}

export const TagDeleteModal: Component<TagDeleteModalProps> = (props) => {
    const { loadTags } = useMetadata();
    const getAllDescendants = (node: TreeNode): number[] => {
        let ids: number[] = [];
        if (node.children) {
            node.children.forEach(child => {
                ids.push(Number(child.id));
                ids = [...ids, ...getAllDescendants(child)];
            });
        }
        return ids;
    };

    const handleConfirm = async () => {
        const node = props.node;
        if (!node) return;

        try {
            const descendantIds = getAllDescendants(node);
            for (const childId of descendantIds) {
                await tagService.deleteTag(childId);
            }
            await tagService.deleteTag(Number(node.id));
            await loadTags();
        } catch (err) {
            console.error("Delete failed:", err);
        } finally {
            props.onClose();
        }
    };

    const count = () => props.node ? getAllDescendants(props.node).length : 0;

    return (
        <ConfirmModal 
            isOpen={props.isOpen}
            onClose={props.onClose}
            onConfirm={handleConfirm}
            title="Delete Tag"
            kind="danger"
            confirmText="Delete"
            message="" // We pass children instead
        >
            <div class="tag-delete-modal-content">
                <p>
                    Are you sure you want to delete tag <strong>"{props.node?.label}"</strong>?
                </p>
                <Show when={count() > 0}>
                    <p class="tag-delete-warning">
                        This will also delete <strong>{count()}</strong> child tags. This action cannot be undone.
                    </p>
                </Show>
            </div>
        </ConfirmModal>
    );
};
