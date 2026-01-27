import { Component, createMemo } from "solid-js";
import { Plus, Pencil, Palette, Trash2 } from "lucide-solid";
import { ContextMenu, ContextMenuItem } from "../../ui/ContextMenu";
import { TreeNode } from "../../ui/TreeView";
import { ColorPicker } from "../../ui/ColorPicker";
import { tagService } from "../../../lib/tags";
import { useMetadata } from "../../../core/hooks";

interface TagContextMenuProps {
    x: number;
    y: number;
    isOpen: boolean;
    node: TreeNode | null;
    onClose: () => void;
    onAddChild: (id: number) => void;
    onRename: (id: number) => void;
    onDelete: (node: TreeNode) => void;
}

export const TagContextMenu: Component<TagContextMenuProps> = (props) => {
    const { loadTags } = useMetadata();
    const items = createMemo<ContextMenuItem[]>(() => {
        const node = props.node;
        if (!node) return [];

        return [
            { 
                type: 'item', 
                label: 'Add Child Tag', 
                icon: Plus, 
                action: () => props.onAddChild(Number(node.id)) 
            },
            { 
                type: 'item', 
                label: 'Rename', 
                icon: Pencil, 
                action: () => props.onRename(Number(node.id)) 
            },
            {
                type: 'submenu', 
                label: 'Change Color', 
                icon: Palette,
                items: [{
                    type: 'custom',
                    content: (
                        <ColorPicker 
                            color={node.iconColor || "#cccccc"} 
                            onChange={async (newColor) => {
                                await tagService.updateTag(Number(node.id), undefined, newColor);
                                await loadTags();
                            }} 
                        />
                    )
                }]
            },
            { type: 'separator' },
            {
                type: 'item', 
                label: 'Delete', 
                danger: true, 
                icon: Trash2,
                action: () => props.onDelete(node)
            }
        ];
    });

    return (
        <ContextMenu 
            x={props.x} 
            y={props.y} 
            items={items()} 
            isOpen={props.isOpen} 
            onClose={props.onClose}
        />
    );
};
