import { Component, JSX } from "solid-js";
import { ReferenceImage } from "./ReferenceImage";
import { type ImageItem } from "../../../types";
import { assetDnD } from "../../../core/dnd";
import { useLibrary, useSelection, useViewport } from "../../../core/hooks";

interface AssetCardProps {
  item: ImageItem;
  style: JSX.CSSProperties | undefined;
  className?: string;
  selected?: boolean;
  onClick?: (e: MouseEvent) => void;
  onDblClick?: (e: MouseEvent) => void;
  onContextMenu?: (e: MouseEvent) => void;
}

// Register directive for this file
 assetDnD;

export const AssetCard: Component<AssetCardProps> = (props) => {
    const lib = useLibrary();
    const selection = useSelection();
    const viewport = useViewport();

    return (
    <div
      use:assetDnD={{ 
          item: props.item, 
          selected: !!props.selected, 
          selectedIds: selection.selectedIds,
          allItems: lib.items
      }}
      class={`virtual-item virtual-masonry-item ${props.selected ? "selected" : ""} ${props.className || ""}`}
      style={{
          ...props.style
      }}
      onClick={props.onClick}
      onDblClick={props.onDblClick || (() => viewport.openItem(props.item.id.toString()))}
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
