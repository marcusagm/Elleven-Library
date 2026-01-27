import { Component } from "solid-js";
import { ConfirmModal } from "../../ui/Modal";
import { TreeNode } from "../../ui/TreeView";
import { tagService } from "../../../lib/tags";
import { appActions } from "../../../core/store/appStore";

interface TagDeleteModalProps {
    isOpen: boolean;
    onClose: () => void;
    node: TreeNode | null;
}

export const TagDeleteModal: Component<TagDeleteModalProps> = (props) => {
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
            await appActions.loadTags();
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
            message={count() > 0 
                ? `Are you sure you want to delete tag "${props.node?.label}" and its ${count()} children? This action cannot be undone.`
                : `Are you sure you want to delete tag "${props.node?.label}"?`}
            kind="danger"
            confirmText="Delete"
        />
    );
};
