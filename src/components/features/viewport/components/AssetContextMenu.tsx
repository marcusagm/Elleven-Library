import { Component, createMemo } from 'solid-js';
import { ExternalLink, FolderOpen, Copy } from 'lucide-solid';
import { ContextMenu, ContextMenuItem } from '../../../ui/ContextMenu';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import { AssetItem } from '../../../../types';

/**
 * Interface for Component's Properties of AssetContextMenu.
 *
 * @param coordinateX - Coordinates X of mouse.
 * @param coordinateY - Coordinates Y of mouse.
 * @param isOpen - State of opening the context menu.
 * @param asset - Asset selected.
 * @param onClose - Function of closing the context menu.
 * @returns {JSX.Element} - Componente AssetContextMenu.
 */
export interface AssetContextMenuProps {
    coordinateX: number;
    coordinateY: number;
    isOpen: boolean;
    asset: AssetItem | null;
    onClose: () => void;
}

/**
 * Renderize the context menu of the asset.
 *
 * @param props - Component's Properties.
 * @returns {JSX.Element} - Component AssetContextMenu.
 */
export const AssetContextMenu: Component<AssetContextMenuProps> = props => {
    const items = createMemo<ContextMenuItem[]>(() => {
        const asset = props.asset;
        if (!asset || !asset.path) return [];

        const menuItems: ContextMenuItem[] = [
            {
                type: 'item',
                label: 'Open file',
                icon: ExternalLink,
                action: async () => {
                    try {
                        await openPath(asset.path);
                    } catch (error) {
                        console.error('Failed to open file:', error);
                    }
                    props.onClose();
                }
            },
            {
                type: 'item',
                label: 'Reveal in OS',
                icon: FolderOpen,
                action: async () => {
                    try {
                        await revealItemInDir(asset.path);
                    } catch (error) {
                        console.error('Failed to reveal file:', error);
                    }
                    props.onClose();
                }
            },
            {
                type: 'separator'
            },
            {
                type: 'item',
                label: 'Copy Path',
                icon: Copy,
                action: async () => {
                    try {
                        await navigator.clipboard.writeText(asset.path);
                    } catch (error) {
                        console.error('Failed to copy path:', error);
                    }
                    props.onClose();
                }
            }
        ];

        return menuItems;
    });

    return (
        <ContextMenu
            coordinateX={props.coordinateX}
            coordinateY={props.coordinateY}
            items={items()}
            isOpen={props.isOpen}
            onClose={props.onClose}
        />
    );
};
