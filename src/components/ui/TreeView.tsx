import { Component, For, createSignal, Show, createEffect } from "solid-js";
import { ChevronRight, ChevronDown } from "lucide-solid";
import { Dynamic } from "solid-js/web";
import { dndRegistry, setDragItem, currentDragItem } from "../../core/dnd";
import "./tree-view.css";

export interface TreeNode {
  id: string | number;
  label: string;
  children?: TreeNode[];
  data?: any;
  icon?: Component<{ size?: number | string; color?: string; fill?: string; stroke?: string }>;
  iconColor?: string;
}

interface TreeViewProps {
  items: TreeNode[];
  onSelect?: (node: TreeNode) => void;
  onContextMenu?: (e: MouseEvent, node: TreeNode) => void;
  selectedId?: string | number;
  editingId?: string | number | null;
  onRename?: (node: TreeNode, newName: string) => void;
  onEditCancel?: () => void;
  defaultIcon?: Component<{ size?: number | string; color?: string; fill?: string; stroke?: string }>;
  // Expansion State
  expandedIds?: Set<string | number>;
  onToggle?: (id: string | number) => void;
  // DnD
  onMove?: (draggedId: string | number, targetId: string | number) => void;
}

interface TreeViewItemProps {
  node: TreeNode;
  depth: number;
  onSelect?: (node: TreeNode) => void;
  onContextMenu?: (e: MouseEvent, node: TreeNode) => void;
  selectedId?: string | number;
  editingId?: string | number | null;
  onRename?: (node: TreeNode, newName: string) => void;
  onEditCancel?: () => void;
  defaultIcon?: Component<{ size?: number | string; color?: string; fill?: string; stroke?: string }>;
  expandedIds?: Set<string | number>;
  onToggle?: (id: string | number) => void;
  onMove?: (draggedId: string | number, targetId: string | number) => void;
}

export const TreeViewItem: Component<TreeViewItemProps> = (props) => {
  // Fallback to local state if no external state provided
  const [localExpanded, setLocalExpanded] = createSignal(false);
  const isExpanded = () => props.expandedIds ? props.expandedIds.has(props.node.id) : localExpanded();
  
  const isEditing = () => props.editingId === props.node.id;
  
  let inputRef: HTMLInputElement | undefined;

  // DnD State
  const [isDragOver, setIsDragOver] = createSignal(false);
  
  const handleDragStart = (e: DragEvent) => {
      e.stopPropagation();
      if (!isEditing() && e.dataTransfer) {
         const data = { type: "TAG", payload: { id: props.node.id } };
         setDragItem(data as any);
         
         e.dataTransfer.effectAllowed = "move";
         e.dataTransfer.setData("application/json", JSON.stringify(data));
      }
  };

  const handleDragEnd = () => {
      setDragItem(null);
      setIsDragOver(false);
  };

  const handleDragEnter = (e: DragEvent) => {
      e.preventDefault(); // Critical to allow dragover
      // We can also check strategy here if we want optimization
  };

  const handleDragOver = (e: DragEvent) => {
      e.preventDefault(); // Critical to allow drop
      const strategy = dndRegistry.get("TAG");
      if (strategy && strategy.onDragOver && currentDragItem && strategy.onDragOver(currentDragItem)) {
          e.dataTransfer!.dropEffect = "move"; 
          setIsDragOver(true);
      }
  };

  const handleDragLeave = (e: DragEvent) => {
      // Check if leaving to child elements?
      // Simple toggle off is usually safe for row items
      setIsDragOver(false);
  };

  const handleDrop = async (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);
      
      try {
          const json = e.dataTransfer?.getData("application/json");
          if (json) {
              const item = JSON.parse(json);
              const strategy = dndRegistry.get("TAG");
              if (strategy && strategy.accepts(item)) {
                  await strategy.onDrop(item, props.node.id);
              }
          }
      } catch (err) {
          console.error("Drop failed", err);
      }
  };

  createEffect(() => {
      if (isEditing() && inputRef) {
          inputRef.focus();
      }
  });

  const hasChildren = () => props.node.children && props.node.children.length > 0;

  const handleToggle = (e: MouseEvent) => {
    e.stopPropagation();
    if (hasChildren()) {
        if (props.onToggle) {
            props.onToggle(props.node.id);
        } else {
            setLocalExpanded(!localExpanded());
        }
    }
  };
  
  const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Enter") {
          (e.currentTarget as HTMLInputElement).blur(); 
      } else if (e.key === "Escape") {
          props.onEditCancel?.();
      }
  };

  const handleBlur = () => {
       if (inputRef) {
            const val = inputRef.value.trim();
            if (val) {
                props.onRename?.(props.node, val);
            } else {
                 props.onEditCancel?.();
            }
       }
  };

  return (
    <div class="tree-item-container">
        <div 
            class={`tree-item-content ${props.selectedId === props.node.id ? 'selected' : ''} ${isDragOver() ? 'drop-target' : ''}`}
            style={{ 
                "padding-left": `${props.depth * 16}px`
            }}
            draggable={!isEditing()}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
            onDragEnter={handleDragEnter}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={() => !isEditing() && props.onSelect?.(props.node)}
            onContextMenu={(e) => {
                e.preventDefault();
                props.onContextMenu?.(e, props.node);
            }}
        >
            <span 
                class={`tree-toggle ${hasChildren() ? 'visible' : ''}`} 
                onClick={handleToggle}
            >
                <Show when={isExpanded()} fallback={<ChevronRight size={14} />}>
                    <ChevronDown size={14} />
                </Show>
            </span>
            
            {/* Icon */}
            <Show when={props.node.icon || props.defaultIcon}>
                <div style={{ 
                    "margin-right": "6px", 
                    display: "flex", 
                    color: props.node.iconColor || "var(--text-secondary)" 
                }}>
                    <Dynamic 
                        component={props.node.icon || props.defaultIcon} 
                        size={14} 
                    />
                </div>
            </Show>

            {/* Label or Input */}
            <Show when={isEditing()} fallback={<span class="tree-label">{props.node.label}</span>}>
                <input 
                    ref={inputRef}
                    type="text" 
                    class="tree-edit-input"
                    value={props.node.label}
                    onClick={(e) => e.stopPropagation()}
                    onKeyDown={handleKeyDown}
                    onBlur={handleBlur}
                    onInput={() => {}}
                />
            </Show>
        </div>
        
        <Show when={isExpanded() && hasChildren()}>
            <For each={props.node.children}>
                {(child) => (
                    <TreeViewItem 
                        node={child} 
                        depth={props.depth + 1} 
                        onSelect={props.onSelect}
                        onContextMenu={props.onContextMenu}
                        selectedId={props.selectedId}
                        editingId={props.editingId}
                        onRename={props.onRename}
                        onEditCancel={props.onEditCancel}
                        defaultIcon={props.defaultIcon}
                        expandedIds={props.expandedIds}
                        onToggle={props.onToggle}
                        onMove={props.onMove}
                    />
                )}
            </For>
        </Show>
    </div>
  );
};

export const TreeView: Component<TreeViewProps> = (props) => {
    return (
        <div class="tree-view">
            <For each={props.items}>
                {(node) => (
                    <TreeViewItem 
                        node={node} 
                        depth={0} 
                        onSelect={props.onSelect} 
                        onContextMenu={props.onContextMenu}
                        selectedId={props.selectedId}
                        editingId={props.editingId}
                        onRename={props.onRename}
                        onEditCancel={props.onEditCancel}
                        defaultIcon={props.defaultIcon}
                        expandedIds={props.expandedIds}
                        onToggle={props.onToggle}
                        onMove={props.onMove}
                    />
                )}
            </For>
        </div>
    );
};
