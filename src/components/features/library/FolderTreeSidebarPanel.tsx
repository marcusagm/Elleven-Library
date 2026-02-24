import { Component, createMemo, createSignal, onMount } from 'solid-js';
import { Folder as FolderIcon, FolderOpen as FolderOpenIcon, Plus } from 'lucide-solid';
import { useMetadata, useFilters, useNotification } from '../../../core/hooks';
import { TreeView, TreeNode } from '../../ui/TreeView';
import { SidebarPanel } from '../../ui/SidebarPanel';
import { Button } from '../../ui/Button';
import { CountBadge } from '../../ui/CountBadge';
import { FolderDeleteModal } from './FolderDeleteModal';
import { FolderContextMenu } from './FolderContextMenu';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './folder-tree-sidebar-panel.css';

/**
 * Metadata for folder nodes to facilitate domain operations.
 */
interface FolderNodeData {
    folderId: number;
    path: string;
    name: string;
    isRoot: boolean;
}

/**
 * Sidebar panel for browsing the file system hierarchy linked to the library.
 */
export const FolderTreeSidebarPanel: Component = () => {
    const metadata = useMetadata();
    const filters = useFilters();
    const notification = useNotification();

    // --- Component State ---
    const [expandedIds, setExpandedIds] = createSignal<Set<string | number>>(new Set());
    const [deleteModalOpen, setDeleteModalOpen] = createSignal(false);
    const [folderToDelete, setFolderToDelete] = createSignal<FolderNodeData | null>(null);
    const [contextMenuOpen, setContextMenuOpen] = createSignal(false);
    const [contextMenuPos, setContextMenuPos] = createSignal({ x: 0, y: 0 });
    const [contextMenuNode, setContextMenuNode] = createSignal<TreeNode | null>(null);

    // --- Lifecycle: Persistence ---
    onMount(() => {
        const savedExpansionState = localStorage.getItem('mundam_folder_expanded');
        if (savedExpansionState) {
            try {
                const parsedIds = JSON.parse(savedExpansionState);
                if (Array.isArray(parsedIds)) {
                    setExpandedIds(new Set(parsedIds));
                }
            } catch (error) {
                console.error('Failed to parse saved folder expansion state:', error);
            }
        }
    });

    const persistExpansionState = (nextSet: Set<string | number>) => {
        setExpandedIds(nextSet);
        localStorage.setItem('mundam_folder_expanded', JSON.stringify(Array.from(nextSet)));
    };

    const toggleExpansion = (id: string | number) => {
        const next = new Set(expandedIds());
        if (next.has(id)) {
            next.delete(id);
        } else {
            next.add(id);
        }
        persistExpansionState(next);
    };

    // --- Tree Construction ---
    const folderTreeHierarchy = createMemo(() => {
        const allFolders = metadata.locations || [];
        const isRecursiveMode = filters.folderRecursiveView;
        const counts = isRecursiveMode
            ? metadata.stats.folder_counts_recursive
            : metadata.stats.folder_counts;

        const nodeMap = new Map<number, TreeNode>();
        const rootNodes: TreeNode[] = [];

        // Phase 1: Create all nodes
        for (const folder of allFolders) {
            nodeMap.set(folder.id, {
                id: `folder-${folder.id}`,
                label: folder.name,
                children: [],
                data: {
                    folderId: folder.id,
                    path: folder.path,
                    name: folder.name,
                    isRoot: folder.is_root
                } as FolderNodeData,
                icon: folder.is_root ? FolderOpenIcon : FolderIcon,
                badge: (
                    <CountBadge
                        showZero={true}
                        count={counts.get(folder.id) || 0}
                        variant="secondary"
                    />
                )
            });
        }

        // Phase 2: Build hierarchy
        for (const folder of allFolders) {
            const node = nodeMap.get(folder.id)!;
            if (folder.parent_id && nodeMap.has(folder.parent_id)) {
                nodeMap.get(folder.parent_id)!.children!.push(node);
            } else {
                rootNodes.push(node);
            }
        }

        return rootNodes.sort((a, b) => a.label.localeCompare(b.label));
    });

    const handleAddFolder = async () => {
        try {
            const selectedPath = await open({
                directory: true,
                multiple: false,
                title: 'Select folder to add to library'
            });

            if (selectedPath) {
                await invoke('add_location', { path: selectedPath });
                await metadata.loadLocations();
                await metadata.loadStats();
                notification.success(
                    'Folder Linked',
                    `Monitoring "${selectedPath.split(/[\\/]/).pop()}"`
                );
            }
        } catch (error) {
            console.error('Failed to add folder:', error);
            notification.error('Failed to Link Folder');
        }
    };

    const handleSelect = (node: TreeNode) => {
        const data = node.data as FolderNodeData;
        filters.setFolder(data.folderId);
    };

    const handleContextMenu = (event: MouseEvent, node: TreeNode) => {
        setContextMenuNode(node);
        setContextMenuPos({ x: event.clientX, y: event.clientY });
        setContextMenuOpen(true);
    };

    // --- Selection State ---
    const activeSelectionIds = createMemo(() => {
        const ids: (string | number)[] = [];
        if (filters.selectedFolderId) {
            ids.push(`folder-${filters.selectedFolderId}`);
        }
        return ids;
    });

    return (
        <>
            <SidebarPanel
                title="Folders"
                class="panel-fluid"
                actions={
                    <Button
                        variant="ghost"
                        size="icon-xs"
                        title="Add Folder"
                        onClick={handleAddFolder}
                    >
                        <Plus size={14} />
                    </Button>
                }
            >
                {folderTreeHierarchy().length > 0 ? (
                    <TreeView
                        items={folderTreeHierarchy()}
                        onSelect={handleSelect}
                        selectedIds={activeSelectionIds()}
                        onContextMenu={handleContextMenu}
                        expandedIds={expandedIds()}
                        onToggle={toggleExpansion}
                        draggable={false}
                        dragType="FOLDER"
                        acceptedDragTypes={[]}
                    />
                ) : (
                    <div class="sidebar-empty-state">
                        <p>No folders linked</p>
                        <p class="empty-hint">Click + to add a folder</p>
                    </div>
                )}
            </SidebarPanel>

            <FolderDeleteModal
                isOpen={deleteModalOpen()}
                onClose={() => setDeleteModalOpen(false)}
                folderId={folderToDelete()?.folderId ?? null}
                folderName={folderToDelete()?.name ?? ''}
            />

            <FolderContextMenu
                isOpen={contextMenuOpen()}
                x={contextMenuPos().x}
                y={contextMenuPos().y}
                node={contextMenuNode()}
                onClose={() => setContextMenuOpen(false)}
                onDelete={node => {
                    setFolderToDelete(node.data as FolderNodeData);
                    setDeleteModalOpen(true);
                }}
            />
        </>
    );
};
