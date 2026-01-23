import { For } from "solid-js";
import { ReferenceImage } from "./ReferenceImage";
import { convertFileSrc } from "@tauri-apps/api/core";

interface ImageItem {
  id: number;
  path: string;
  filename: string;
  src: string;
  width: number | null;
  height: number | null;
  thumbnail_path: string | null;
}

interface ImageGridProps {
  items: ImageItem[];
}

export function ImageGrid(props: ImageGridProps) {
  return (
    <div class="masonry-grid">
      <For each={props.items}>
        {(item) => (
          <div class="masonry-item">
            <ReferenceImage 
              src={item.path} 
              thumbnail={item.thumbnail_path}
              alt={item.filename} 
              width={item.width}
              height={item.height}
            />
            <div class="item-overlay">
              <span class="item-name">{item.filename}</span>
            </div>
          </div>
        )}
      </For>
    </div>
  );
}

