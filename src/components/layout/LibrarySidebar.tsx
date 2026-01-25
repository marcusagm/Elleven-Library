import { Component, For } from "solid-js";
import { useAppStore } from "../../core/store/appStore";
import { Library, Folder, Trash2, Tag, Layers } from "lucide-solid";

export const LibrarySidebar: Component = () => {
  const { state } = useAppStore();

  return (
    <div style={{ padding: "8px" }}>
        {/* Section Header */}
        <div style={{ 
            padding: "16px 8px 8px 8px", 
            "font-size": "11px", 
            "font-weight": "600", 
            color: "var(--text-muted)", 
            "text-transform": "uppercase",
            "letter-spacing": "0.5px"
        }}>
            Library
        </div>
        
        {/* Core Filters */}
        <div class="nav-item active">
            <Layers size={16} />
            <span style={{ flex: 1 }}>All Items</span>
            <span style={{ opacity: 0.5, "font-size": "10px" }}>{state.items.length}</span>
        </div>
        <div class="nav-item">
            <Tag size={16} />
            <span>Uncategorized</span>
        </div>
        <div class="nav-item">
            <Trash2 size={16} />
            <span>Trash</span>
        </div>

        {/* Folders Section */}
        <div style={{ 
            padding: "24px 8px 8px 8px", 
            "font-size": "11px", 
            "font-weight": "600", 
            color: "var(--text-muted)", 
            "text-transform": "uppercase",
            "letter-spacing": "0.5px",
            display: "flex", "align-items": "center", "justify-content": "space-between"
        }}>
            <span>Folders</span>
            <button style={{ background: "none", border: "none", color: "inherit", cursor: "pointer" }}>+</button>
        </div>
        
        {/* Folder List */}
        <div style={{ "display": "flex", "flex-direction": "column", gap: "1px" }}>
            <For each={state.locations}>
                {(loc) => (
                    <div class="nav-item" title={loc.path}>
                        <Folder size={16} fill="var(--text-muted)" stroke="none" /> 
                        <span style={{ "white-space": "nowrap", "overflow": "hidden", "text-overflow": "ellipsis" }}>
                            {loc.name}
                        </span>
                    </div>
                )}
            </For>
            
            {state.locations.length === 0 && (
                <div style={{ padding: "12px", "text-align": "center", color: "var(--text-muted)", "font-size": "11px", "font-style": "italic" }}>
                    No folders linked
                </div>
            )}
        </div>
    </div>
  );
};
