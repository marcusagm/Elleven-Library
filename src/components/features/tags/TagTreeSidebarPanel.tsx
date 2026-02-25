import { Component, createMemo, createSignal, onMount } from 'solid-js';
import { Tag as TagIcon, Plus } from 'lucide-solid';
import { useMetadata, useFilters, useNotification } from '../../../core/hooks';
import { TreeView, TreeNode } from '../../ui/TreeView';
import { SidebarPanel } from '../../ui/SidebarPanel';
import { Button } from '../../ui/Button';
import { CountBadge } from '../../ui/CountBadge';
import { dndRegistry, setDragItem, DragItem } from '../../../core/dnd';
import { tagService } from '../../../lib/tags';
import { TagContextMenu } from './TagContextMenu';
import { TagDeleteModal } from './TagDeleteModal';

/**
 * Sidebar panel for managing the hierarchical tag system.
 * Integrates metadata state with the pure TreeView component.
 */
export const TagTreeSidebarPanel: Component = () => {
    const metadata = useMetadata();
    const filters = useFilters();
    const notification = useNotification();
    const [isTagHeaderDragOver, setIsTagHeaderDragOver] = createSignal(false);

    // --- Component State ---
    const [contextMenuOpen, setContextMenuOpen] = createSignal(false);
    const [contextMenuPosition, setContextMenuPosition] = createSignal({
        coordinateX: 0,
        coordinateY: 0
    });
    const [contextMenuNode, setContextMenuNode] = createSignal<TreeNode | null>(null);

    const [editingId, setEditingId] = createSignal<number | null>(null);
    const [expandedIds, setExpandedIds] = createSignal<Set<string | number>>(new Set());

    const [deleteModalOpen, setDeleteModalOpen] = createSignal(false);
    const [nodeToDelete, setNodeToDelete] = createSignal<TreeNode | null>(null);

    // --- Lifecycle: Persistence ---
    onMount(() => {
        const savedExpansionState = localStorage.getItem('mundam_tree_expanded');
        if (savedExpansionState) {
            try {
                const parsedIds = JSON.parse(savedExpansionState);
                if (Array.isArray(parsedIds)) {
                    setExpandedIds(new Set(parsedIds));
                }
            } catch (error) {
                console.error('Failed to parse saved expansion state:', error);
            }
        }
    });

    const persistExpansionState = (nextSet: Set<string | number>) => {
        setExpandedIds(nextSet);
        localStorage.setItem('mundam_tree_expanded', JSON.stringify(Array.from(nextSet)));
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

    const expandNode = (id: string | number) => {
        if (expandedIds().has(id)) return;
        const next = new Set(expandedIds());
        next.add(id);
        persistExpansionState(next);
    };

    // --- Tree Construction ---
    const tagTreeHierarchy = createMemo(() => {
        const allTags = metadata.tags || [];
        const nodeMap = new Map<number, TreeNode>();
        const rootNodes: TreeNode[] = [];

        // Phase 1: Create all nodes
        allTags.forEach(tag => {
            nodeMap.set(tag.id, {
                id: tag.id,
                label: tag.name,
                children: [],
                data: tag,
                icon: TagIcon,
                iconColor: tag.color || undefined,
                badge: (
                    <CountBadge
                        showZero={true}
                        count={metadata.stats.tag_counts.get(tag.id) || 0}
                    />
                )
            });
        });

        // Phase 2: Build hierarchy relationships
        allTags.forEach(tag => {
            if (tag.parent_id && nodeMap.has(tag.parent_id)) {
                nodeMap.get(tag.parent_id)!.children!.push(nodeMap.get(tag.id)!);
            } else {
                rootNodes.push(nodeMap.get(tag.id)!);
            }
        });

        return rootNodes;
    });

    // --- Domain Logic Handlers ---
    const getUniqueTagName = (baseName: string) => {
        const existingTags = metadata.tags || [];
        let name = baseName;
        let counter = 1;
        while (existingTags.some(tag => tag.name === name)) {
            name = `${baseName} (${counter})`;
            counter++;
        }
        return name;
    };

    const handleCreateTag = async () => {
        if (editingId() !== null) return;
        try {
            const name = getUniqueTagName('New Tag');
            const newTagId = await tagService.createTag(name);
            await metadata.loadTags();
            setEditingId(newTagId);
        } catch (error) {
            console.error('Failed to create tag:', error);
            notification.error('Failed to Create Tag');
        }
    };

    const handleCreateChildTag = async (parentId: number) => {
        try {
            expandNode(parentId);
            const name = getUniqueTagName('New Tag');
            const newTagId = await tagService.createTag(name, parentId);
            await metadata.loadTags();
            setEditingId(newTagId);
        } catch (error) {
            console.error('Failed to create child tag:', error);
            notification.error('Failed to Create Child Tag');
        }
    };

    const handleRenameTag = async (node: TreeNode, newName: string) => {
        if (!newName || !newName.trim() || newName === node.label) {
            setEditingId(null);
            return;
        }

        const oldName = node.label;
        const isPlaceholder = /^New Tag( \(\d+\))?$/.test(oldName);

        try {
            await tagService.updateTag(Number(node.id), newName);
            await metadata.loadTags();

            if (isPlaceholder) {
                notification.success('Tag Created', `Created tag "${newName}"`);
            } else {
                notification.success('Tag Renamed', `Changed "${oldName}" to "${newName}"`);
            }
        } catch (error) {
            console.error('Failed to rename tag:', error);
            notification.error('Failed to Rename Tag');
        } finally {
            setEditingId(null);
        }
    };

    /**
     * Triggers the context menu for a specific node.
     *
     * @param {MouseEvent} event - The mouse event triggered by right-click.
     * @param {TreeNode} node - The tree node being interacted with.
     */
    const handleContextMenu = (event: MouseEvent, node: TreeNode) => {
        event.preventDefault();
        setContextMenuPosition({ coordinateX: event.clientX, coordinateY: event.clientY });
        setContextMenuNode(node);
        setContextMenuOpen(true);
    };

    /**
     * Domain-specific validation for tag drops.
     * Prevents dropping a tag into its own descendant hierarchy.
     */
    const isValidTagDrop = (draggedItem: DragItem, targetNode: TreeNode): boolean => {
        if (draggedItem.type !== 'TAG') return true; // Images can be dropped anywhere

        const draggedId = Number(draggedItem.payload.id);
        const targetId = Number(targetNode.id);

        /** Recursive check for descendants */
        const isDescendant = (node: TreeNode, searchId: number): boolean => {
            if (node.children) {
                for (const child of node.children) {
                    if (Number(child.id) === searchId) return true;
                    if (isDescendant(child, searchId)) return true;
                }
            }
            return false;
        };

        // Find the dragged node in our local tree to check its children
        const findDraggedNode = (nodes: TreeNode[]): TreeNode | null => {
            for (const node of nodes) {
                if (Number(node.id) === draggedId) return node;
                if (node.children) {
                    const found = findDraggedNode(node.children);
                    if (found) return found;
                }
            }
            return null;
        };

        const node = findDraggedNode(tagTreeHierarchy());
        return !node || !isDescendant(node, targetId);
    };

    /**
     * Handles drops on the root area of the tag panel.
     * Allows tags to be moved back to the root level.
     *
     * @param {DragEvent} event - The browser drop event.
     */
    const handleRootDrop = async (event: DragEvent) => {
        event.preventDefault();
        setIsTagHeaderDragOver(false);
        try {
            const rawJsonData = event.dataTransfer?.getData('application/json');
            if (rawJsonData) {
                const droppedItem: DragItem = JSON.parse(rawJsonData);
                const tagStrategy = dndRegistry.get('TAG');
                if (tagStrategy && droppedItem.type === 'TAG') {
                    await tagStrategy.onDrop(droppedItem, 'root');
                }
            }
        } catch (error) {
            console.error('Root tag drop failed:', error);
        }
        setDragItem(null);
    };

    return (
        <SidebarPanel
            title="Tags"
            class="panel-fluid"
            actions={
                <Button variant="ghost" size="icon-xs" title="Create Tag" onClick={handleCreateTag}>
                    <Plus size={14} />
                </Button>
            }
            contentClass={isTagHeaderDragOver() ? 'drag-over' : ''}
            onDragOver={(event: DragEvent) => {
                const strategy = dndRegistry.get('TAG');
                if (strategy) {
                    event.preventDefault();
                    if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
                    setIsTagHeaderDragOver(true);
                }
            }}
            onDragLeave={() => setIsTagHeaderDragOver(false)}
            onDrop={handleRootDrop}
        >
            <TreeView
                items={tagTreeHierarchy()}
                onSelect={node => filters.toggleTag(Number(node.id))}
                selectedIds={filters.selectedTags}
                onContextMenu={handleContextMenu}
                editingId={editingId()}
                onRename={handleRenameTag}
                onEditCancel={() => setEditingId(null)}
                expandedIds={expandedIds()}
                onToggle={toggleExpansion}
                draggable={true}
                dragType="TAG"
                acceptedDragTypes={['TAG', 'IMAGE']}
                isValidDrop={isValidTagDrop}
            />

            <TagContextMenu
                coordinateX={contextMenuPosition().coordinateX}
                coordinateY={contextMenuPosition().coordinateY}
                isOpen={contextMenuOpen()}
                node={contextMenuNode()}
                onClose={() => setContextMenuOpen(false)}
                onAddChild={handleCreateChildTag}
                onRename={id => setEditingId(id)}
                onDelete={node => {
                    setNodeToDelete(node);
                    setDeleteModalOpen(true);
                }}
            />

            <TagDeleteModal
                isOpen={deleteModalOpen()}
                onClose={() => setDeleteModalOpen(false)}
                node={nodeToDelete()}
            />
        </SidebarPanel>
    );
};
