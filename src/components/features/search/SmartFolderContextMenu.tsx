import { Component, createMemo } from 'solid-js';
import { Edit, Trash2 } from 'lucide-solid';
import { ContextMenu, ContextMenuItem } from '../../ui/ContextMenu';
import { SmartFolder } from '../../../core/store/metadata';

/**
 * Properties for the SmartFolderContextMenu component.
 */
interface SmartFolderContextMenuProperties {
    /** The X screen coordinate where the menu should appear. */
    coordinateX: number;
    /** The Y screen coordinate where the menu should appear. */
    coordinateY: number;
    /** Whether the menu is currently visible. */
    isOpen: boolean;
    /** The smart folder metadata for which the menu is displayed. */
    folder: SmartFolder | null;
    /** Callback invoked when the menu requests closure. */
    onClose: () => void;
    /** Callback invoked to edit the folder. */
    onEdit: (folder: SmartFolder) => void;
    /** Callback invoked to delete the folder. */
    onDelete: (folder: SmartFolder) => void;
}

/**
 * Context menu providing management actions for a smart folder.
 *
 * @param componentProperties - Properties for the component.
 * @returns The rendered SmartFolderContextMenu.
 */
export const SmartFolderContextMenu: Component<
    SmartFolderContextMenuProperties
> = componentProperties => {
    const contextMenuItems = createMemo<ContextMenuItem[]>(() => {
        const folder = componentProperties.folder;
        if (!folder) {
            return [];
        }

        return [
            {
                type: 'item',
                label: 'Edit Smart Folder',
                icon: Edit,
                action: () => componentProperties.onEdit(folder)
            },
            { type: 'separator' },
            {
                type: 'item',
                label: 'Delete',
                danger: true,
                icon: Trash2,
                action: () => componentProperties.onDelete(folder)
            }
        ];
    });

    return (
        <ContextMenu
            coordinateX={componentProperties.coordinateX}
            coordinateY={componentProperties.coordinateY}
            items={contextMenuItems()}
            isOpen={componentProperties.isOpen}
            onClose={componentProperties.onClose}
        />
    );
};
