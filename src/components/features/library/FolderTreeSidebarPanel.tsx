import { Button } from '../../ui';
import { Component, createMemo, createSignal, onMount } from 'solid-js';
import { Folder as FolderIcon, FolderOpen as FolderOpenIcon, Plus } from 'lucide-solid';
import { useMetadata, useFilters, useNotification } from '../../../core/hooks';
import { TreeView, TreeNode } from '../../ui/TreeView';
import { SidebarPanel } from '../../ui/SidebarPanel';
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
    /** The unique identifier of the folder. */
    folderIdentifier: number;
    /** The full file system path. */
    path: string;
    /** The display name of the folder. */
    name: string;
    /** Whether this is a root-level monitored folder. */
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
    const [expandedIdentifiers, setExpandedIdentifiers] = createSignal<Set<string | number>>(
        new Set()
    );
    const [deleteModalOpen, setDeleteModalOpen] = createSignal(false);
    const [folderToDelete, setFolderToDelete] = createSignal<FolderNodeData | null>(null);
    const [contextMenuOpen, setContextMenuOpen] = createSignal(false);
    const [contextMenuPosition, setContextMenuPosition] = createSignal({
        coordinateX: 0,
        coordinateY: 0
    });
    const [contextMenuNode, setContextMenuNode] = createSignal<TreeNode | null>(null);

    // --- Lifecycle: Persistence ---
    onMount(() => {
        const savedExpansionState = localStorage.getItem('mundam_folder_expanded');
        if (savedExpansionState) {
            try {
                const parsedIdentifiers = JSON.parse(savedExpansionState);
                if (Array.isArray(parsedIdentifiers)) {
                    setExpandedIdentifiers(new Set(parsedIdentifiers));
                }
            } catch (error) {
                console.error('Failed to parse saved folder expansion state:', error);
            }
        }
    });

    /**
     * Persists the current tree expansion state to local storage.
     *
     * @param nextSet - The new set of expanded node identifiers.
     */
    const persistExpansionState = (nextSet: Set<string | number>) => {
        setExpandedIdentifiers(nextSet);
        localStorage.setItem('mundam_folder_expanded', JSON.stringify(Array.from(nextSet)));
    };

    /**
     * Toggles the expansion state of a specific node.
     *
     * @param identifier - The identifier of the node to toggle.
     */
    const toggleExpansion = (identifier: string | number) => {
        const nextSet = new Set(expandedIdentifiers());
        if (nextSet.has(identifier)) {
            nextSet.delete(identifier);
        } else {
            nextSet.add(identifier);
        }
        persistExpansionState(nextSet);
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
                    folderIdentifier: folder.id,
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
            const node = nodeMap.get(folder.id);
            if (!node) {
                continue;
            }

            if (folder.parent_id && nodeMap.has(folder.parent_id)) {
                nodeMap.get(folder.parent_id)!.children!.push(node);
            } else {
                rootNodes.push(node);
            }
        }

        return rootNodes.sort((a, b) => a.label.localeCompare(b.label));
    });

    /**
     * Opens a directory picker to add a new monitored folder to the library.
     */
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

    /**
     * Handles the selection of a folder node.
     *
     * @param node - The selected tree node.
     */
    const handleSelect = (node: TreeNode) => {
        const data = node.data as FolderNodeData;
        filters.setFolder(data.folderIdentifier);
    };

    /**
     * Triggers the context menu for a specific node.
     *
     * @param event - The mouse event triggered by right-click.
     * @param node - The tree node being interacted with.
     */
    const handleContextMenu = (event: MouseEvent, node: TreeNode) => {
        event.preventDefault();
        setContextMenuNode(node);
        setContextMenuPosition({ coordinateX: event.clientX, coordinateY: event.clientY });
        setContextMenuOpen(true);
    };

    // --- Selection State ---
    const activeSelectionIdentifiers = createMemo(() => {
        const identifiers: (string | number)[] = [];
        if (filters.selectedFolderId) {
            identifiers.push(`folder-${filters.selectedFolderId}`);
        }
        return identifiers;
    });

    return (
        <>
            <SidebarPanel
                title="Folders"
                class="panel-fluid"
                headerActions={
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
                        selectedIds={activeSelectionIdentifiers()}
                        onContextMenu={handleContextMenu}
                        expandedIds={expandedIdentifiers()}
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
                folderIdentifier={folderToDelete()?.folderIdentifier ?? null}
                folderName={folderToDelete()?.name ?? ''}
            />

            <FolderContextMenu
                isOpen={contextMenuOpen()}
                coordinateX={contextMenuPosition().coordinateX}
                coordinateY={contextMenuPosition().coordinateY}
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
