import { Component, createMemo } from 'solid-js';
import { Plus, Pencil, Palette, Trash2 } from 'lucide-solid';
import { ContextMenu, ContextMenuItem } from '../../ui/ContextMenu';
import { TreeNode } from '../../ui/TreeView';
import { ColorPicker } from '../../ui/ColorPicker';
import { useMetadata } from '../../../core/hooks';

interface TagContextMenuProps {
    coordinateX: number;
    coordinateY: number;
    isOpen: boolean;
    node: TreeNode | null;
    onClose: () => void;
    onAddChild: (id: number) => void;
    onRename: (id: number) => void;
    onDelete: (node: TreeNode) => void;
}

/**
 * Context menu for tag tree nodes.
 * Provides actions for adding, renaming, deleting, and changing tag colors.
 *
 * @param {TagContextMenuProps} properties - Component properties.
 * @returns {import('solid-js').JSX.Element} The rendered context menu.
 */
export const TagContextMenu: Component<TagContextMenuProps> = properties => {
    const metadata = useMetadata();

    /**
     * Updates a tag's color and refreshes the metadata.
     *
     * @param {number} tagId - The unique identifier of the tag.
     * @param {string} newColor - The new hexadecimal color code.
     */
    const handleColorChange = async (tagId: number, newColor: string) => {
        // Explicitly pass null for name to indicate no change, or let undefined handle it.
        // The store action will pass this to tagService.
        await metadata.updateTag(tagId, null, newColor);
    };

    const contextMenuItems = createMemo<ContextMenuItem[]>(() => {
        const treeNode = properties.node;
        if (!treeNode) return [];

        return [
            {
                type: 'item',
                label: 'Add Child Tag',
                icon: Plus,
                action: () => properties.onAddChild(Number(treeNode.id))
            },
            {
                type: 'item',
                label: 'Rename',
                icon: Pencil,
                action: () => properties.onRename(Number(treeNode.id))
            },
            {
                type: 'submenu',
                label: 'Change Color',
                icon: Palette,
                items: [
                    {
                        type: 'custom',
                        content: (
                            <ColorPicker
                                color={treeNode.iconColor || '#cccccc'}
                                onChange={newColor =>
                                    handleColorChange(Number(treeNode.id), newColor)
                                }
                            />
                        )
                    }
                ]
            },
            { type: 'separator' },
            {
                type: 'item',
                label: 'Delete',
                danger: true,
                icon: Trash2,
                action: () => properties.onDelete(treeNode)
            }
        ];
    });

    return (
        <ContextMenu
            coordinateX={properties.coordinateX}
            coordinateY={properties.coordinateY}
            items={contextMenuItems()}
            isOpen={properties.isOpen}
            onClose={properties.onClose}
        />
    );
};
