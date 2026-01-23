import { createSignal, createEffect, onMount, onCleanup, For } from "solid-js";
import { ReferenceImage } from "./ReferenceImage";

interface ImageItem {
  id: number;
  path: string;
  filename: string;
  width: number | null;
  height: number | null;
  thumbnail_path: string | null;
}

interface VirtualMasonryProps {
  items: ImageItem[];
}

export function VirtualMasonry(props: VirtualMasonryProps) {
  let scrollContainer: HTMLDivElement | undefined;
  
  const [columns, setColumns] = createSignal(4);
  const [containerWidth, setContainerWidth] = createSignal(0);
  const [scrollTop, setScrollTop] = createSignal(0);
  const [containerHeight, setContainerHeight] = createSignal(0);
  
  // Layout state
  const [layout, setLayout] = createSignal<{ height: number; positions: Map<number, { x: number; y: number; width: number; height: number }> }>({ height: 0, positions: new Map() });
  
  // Configuration
  const minColWidth = 280;
  const gap = 16; // var(--space-4)
  
  // 1. Handle resize to update column count
  onMount(() => {
    if (!scrollContainer) return;
    
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const width = entry.contentRect.width;
        setContainerWidth(width);
        
        // Calculate columns
        // width = cols * colWidth + (cols - 1) * gap
        // Using minColWidth as a basis
        const calculatedCols = Math.max(1, Math.floor((width + gap) / (minColWidth + gap)));
        setColumns(calculatedCols);
        setContainerHeight(entry.contentRect.height);
      }
    });
    
    observer.observe(scrollContainer);
    
    // Handle scroll
    const handleScroll = () => {
      setScrollTop(scrollContainer?.scrollTop || 0);
    };
    
    scrollContainer.addEventListener("scroll", handleScroll, { passive: true });
    
    onCleanup(() => {
      observer.disconnect();
      scrollContainer?.removeEventListener("scroll", handleScroll);
    });
  });

  // 2. Calculate layout whenever items or cols change
  createEffect(() => {
    const items = props.items;
    const cols = columns();
    const width = containerWidth();
    
    // Only recalc if we have width and items
    if (width === 0 || items.length === 0) return;
    
    // Defer calc to next microtask to avoid resize loop
    requestAnimationFrame(() => {
        const totalGapWidth = (cols - 1) * gap;
        const colWidth = (width - totalGapWidth) / cols;
        
        const colHeights = new Array(cols).fill(0);
        const newPositions = new Map();
        
        items.forEach((item) => {
          let minH = colHeights[0];
          let colIdx = 0;
          for (let i = 1; i < cols; i++) {
            if (colHeights[i] < minH) {
              minH = colHeights[i];
              colIdx = i;
            }
          }
          
          const x = colIdx * (colWidth + gap);
          const y = minH;
          
          const aspectRatio = (item.width && item.height) ? item.width / item.height : 1;
          const h = colWidth / aspectRatio;
          
          newPositions.set(item.id, { x, y, width: colWidth, height: h });
          
          colHeights[colIdx] += h + gap;
        });
        
        const maxHeight = Math.max(...colHeights);
        setLayout({ height: maxHeight, positions: newPositions });
    });
  });
  
  // 3. Determine visible items
  // Memoize this calculation based on scroll, layout, and items
  const visibleItems = () => {
    const sTop = scrollTop();
    const vHeight = containerHeight();
    const buffer = 1000; // Render buffer
    const currentLayout = layout();
    
    // Safety check: if layout is stale (no positions), return empty to avoid glitch
    if (currentLayout.positions.size === 0 && props.items.length > 0) return [];

    return props.items.filter(item => {
      const pos = currentLayout.positions.get(item.id);
      if (!pos) return false;
      
      return (pos.y + pos.height > sTop - buffer) && (pos.y < sTop + vHeight + buffer);
    });
  };

  return (
    <div 
      ref={scrollContainer} 
      class="virtual-scroll-container" 
      style={{ 
        width: "100%", 
        height: "100%", 
        "overflow-y": "auto", 
        position: "relative",
        "will-change": "transform" // Promote to layer
      }}
    >
      <div 
        class="virtual-track" 
        style={{ 
          height: `${layout().height}px`, 
          width: "100%", 
          position: "relative" 
        }}
      >
        <For each={visibleItems()}>
          {(item) => {
            const pos = layout().positions.get(item.id)!;
            return (
              <div
                class="virtual-item masonry-item" // Re-use masonry-item for styling but override positioning
                style={{
                  position: "absolute",
                  transform: `translate3d(${pos.x}px, ${pos.y}px, 0)`,
                  width: `${pos.width}px`,
                  height: `${pos.height}px`,
                  "margin-bottom": "0" // Override CSS
                }}
              >
                <ReferenceImage
                  id={item.id}
                  src={item.path}
                  thumbnail={item.thumbnail_path}
                  alt={item.filename}
                  width={item.width}
                  height={item.height}
                />
                 <div class="item-overlay">
                  <span class="item-name">#{item.id} - {item.filename}</span>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
}
