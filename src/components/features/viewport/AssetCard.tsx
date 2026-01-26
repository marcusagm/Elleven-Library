import { Component, JSX, createSignal } from "solid-js";
import { ReferenceImage } from "./ReferenceImage";
import { type ImageItem } from "../../../utils/masonryLayout";
import { dndRegistry, setDragItem, currentDragItem } from "../../../core/dnd";
import { useAppStore } from "../../../core/store/appStore"; // Import store

interface AssetCardProps {
  item: ImageItem;
  style: JSX.CSSProperties | undefined;
  className?: string;
  selected?: boolean;
  onClick?: (e: MouseEvent) => void;
  onContextMenu?: (e: MouseEvent) => void;
}

export const AssetCard: Component<AssetCardProps> = (props) => {
  const { state } = useAppStore();
  const [isDropTarget, setIsDropTarget] = createSignal(false);
  
  const handleDragStart = (e: DragEvent) => {
    e.stopPropagation();
    if (!e.dataTransfer) return;
    
    // Determine selection
    let ids = [props.item.id];
    const selectedIds = state.selection;
    
    if (props.selected && selectedIds.includes(props.item.id)) {
        // If dragging a selected item, include ALL selected items
        ids = [...selectedIds];
    }
    
    const data = { type: "IMAGE", payload: { ids } };
    
    // Set Global State
    setDragItem(data as any);
    
    e.dataTransfer.effectAllowed = "copyMove";
    e.dataTransfer.setData("application/json", JSON.stringify(data));
    
    // Native File Paths
    if (ids.length === 1 && props.item.path) {
        e.dataTransfer.setData("text/uri-list", `file://${props.item.path}`);
        e.dataTransfer.setData("text/plain", props.item.path);
    } else if (ids.length > 0) {
        const paths = ids.map(id => {
            const item = state.items.find(i => i.id === id);
            return item ? `file://${item.path}` : null;
        }).filter(Boolean).join("\r\n");
        e.dataTransfer.setData("text/uri-list", paths);
    }
  };
  
  const handleDragEnd = () => {
      setDragItem(null);
      setIsDropTarget(false);
  };

  const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
  };

  const handleDragOver = (e: DragEvent) => {
      e.preventDefault(); 
      const strategy = dndRegistry.get("IMAGE");
      // Use global currentDragItem for validation
      if (strategy && strategy.onDragOver && currentDragItem && strategy.onDragOver(currentDragItem)) {
           e.dataTransfer!.dropEffect = "copy";
           setIsDropTarget(true);
      }
  };
  
  const handleDragLeave = () => setIsDropTarget(false);

  const handleDrop = async (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDropTarget(false);
      
      try {
          const json = e.dataTransfer?.getData("application/json");
          if (json) {
              const item = JSON.parse(json);
              const strategy = dndRegistry.get("IMAGE");
              if (strategy && strategy.accepts(item)) {
                  await strategy.onDrop(item, props.item.id);
              }
          }
      } catch (err) {
          console.error("Drop failed", err);
      }
  };

  return (
    <div
      class={`virtual-item virtual-masonry-item ${props.selected ? "selected" : ""} ${props.className || ""} ${isDropTarget() ? "drop-target-active" : ""}`}
      style={{
          ...props.style,
          "border": isDropTarget() ? "2px solid var(--primary)" : undefined
      }}
      draggable={true}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragEnter={handleDragEnter}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onClick={props.onClick}
      onContextMenu={props.onContextMenu}
    >
        <div style={{ width: "100%", height: "100%", "pointer-events": "none" }}>
          <ReferenceImage
            id={props.item.id}
            src={props.item.path}
            thumbnail={props.item.thumbnail_path}
            alt={props.item.filename}
            width={props.item.width}
            height={props.item.height}
          />
          
          <div class="item-overlay">
            <span class="item-name">#{props.item.id} - {props.item.filename}</span>
          </div>
        </div>
    </div>
  );
};
