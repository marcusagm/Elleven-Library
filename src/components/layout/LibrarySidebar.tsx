import { Component, For } from "solid-js";
import { useAppStore } from "../../core/store/appStore";
import { Folder, Trash2, Tag, Layers, Plus } from "lucide-solid";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Separator } from "../ui/Separator";

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
            <Badge variant="secondary">{state.items.length}</Badge>
        </div>
        <div class="nav-item">
            <Tag size={16} />
            <span>Uncategorized</span>
        </div>
        <div class="nav-item">
            <Trash2 size={16} />
            <span>Trash</span>
        </div>

        <Separator class="my-3" style={{ margin: "12px 8px" }} />

        {/* Folders Section */}
        <div style={{ 
            padding: "8px", 
            "font-size": "11px", 
            "font-weight": "600", 
            color: "var(--text-muted)", 
            "text-transform": "uppercase",
            "letter-spacing": "0.5px",
            display: "flex", "align-items": "center", "justify-content": "space-between"
        }}>
            <span>Folders</span>
            <Button variant="ghost" size="icon-sm" title="Create Folder">
                <Plus size={14} />
            </Button>
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
