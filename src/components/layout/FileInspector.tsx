import { Component, Show, createMemo } from "solid-js";
import { useAppStore } from "../../core/store/appStore";
import { Info, Tag, Hash, Calendar, Maximize2 } from "lucide-solid";

export const FileInspector: Component = () => {
  const { state } = useAppStore();

  // Derived state for the active item
  const activeItem = createMemo(() => {
    if (state.selection.length === 0) return null;
    const id = state.selection[state.selection.length - 1];
    return state.items.find((i) => i.id === id);
  });

  return (
    <div style={{ height: "100%", display: "flex", "flex-direction": "column" }}>
        {/* Header */}
        <div style={{ 
             padding: "16px 12px 12px", 
             "font-size": "11px", 
             "font-weight": "600", 
             color: "var(--text-muted)", 
             "text-transform": "uppercase",
             "letter-spacing": "0.5px",
             "border-bottom": "1px solid var(--border-subtle)"
        }}>
            Inspector
        </div>

        <Show when={activeItem()} fallback={
            <div style={{ 
                flex: 1, 
                display: "flex", 
                "flex-direction": "column",
                "align-items": "center", 
                "justify-content": "center", 
                color: "var(--text-muted)", 
                gap: "12px",
                opacity: 0.5
            }}>
                <Info size={32} />
                <span style={{ "font-size": "12px" }}>No item selected</span>
            </div>
        }>
            <div style={{ padding: "16px", "overflow-y": "auto" }}>
                {/* Preview */}
                <div style={{ "margin-bottom": "20px" }}>
                    <div class="reference-image-container" style={{ 
                        "aspect-ratio": "1/1", 
                        "border-radius": "var(--radius-md)", 
                        "background": "var(--bg-color)",
                        "border": "1px solid var(--border-subtle)",
                        "display": "flex",
                        "align-items": "center",
                        "justify-content": "center",
                        "overflow": "hidden"
                    }}>
                       <img 
                            src={activeItem()?.thumbnail_path ? `thumb://localhost/${activeItem()?.thumbnail_path?.split(/[\\/]/).pop()}` : ""} 
                            style={{ "max-width": "100%", "max-height": "100%", "object-fit": "contain" }} 
                       />
                    </div>
                </div>

                {/* Fields */}
                <div class="field-group" style={{ "margin-bottom": "16px" }}>
                    <label style={{ display: "block", "font-size": "11px", color: "var(--text-secondary)", "margin-bottom": "6px", "font-weight": "500" }}>Name</label>
                    <input 
                        type="text" 
                        value={activeItem()?.filename} 
                        disabled 
                        style={{ 
                            width: "100%", 
                            background: "var(--bg-color)",
                            border: "1px solid var(--border-color)",
                            padding: "8px",
                            "border-radius": "var(--radius-sm)",
                            color: "var(--text-primary)",
                            "font-size": "12px"
                        }} 
                    />
                </div>

                <div class="field-group" style={{ "margin-bottom": "16px" }}>
                    <label style={{ display: "flex", "align-items": "center", gap: "6px", "font-size": "11px", color: "var(--text-secondary)", "margin-bottom": "6px", "font-weight": "500" }}>
                        <Tag size={12} /> Tags
                    </label>
                    <div style={{ 
                        display: "flex", 
                        "flex-wrap": "wrap", 
                        gap: "6px", 
                        padding: "8px", 
                        background: "var(--bg-color)", 
                        border: "1px dashed var(--border-color)", 
                        "border-radius": "var(--radius-sm)",
                        "min-height": "36px"
                    }}>
                        <span style={{ "font-size": "11px", color: "var(--text-muted)", "font-style": "italic" }}>Add tags...</span>
                    </div>
                </div>

                {/* Metadata Grid */}
                <div style={{ 
                    "margin-top": "24px", 
                    "border-top": "1px solid var(--border-subtle)", 
                    "padding-top": "16px",
                    display: "grid",
                    "grid-template-columns": "1fr 1fr",
                    gap: "12px"
                }}>
                    <div>
                        <span style={{ display: "block", "font-size": "10px", color: "var(--text-muted)", "margin-bottom": "2px" }}>DIMENSIONS</span>
                        <div style={{ display: "flex", "align-items": "center", gap: "6px", "font-size": "12px", color: "var(--text-secondary)" }}>
                            <Maximize2 size={12} />
                            {activeItem()?.width} x {activeItem()?.height}
                        </div>
                    </div>
                    <div>
                        <span style={{ display: "block", "font-size": "10px", color: "var(--text-muted)", "margin-bottom": "2px" }}>TYPE</span>
                         <div style={{ display: "flex", "align-items": "center", gap: "6px", "font-size": "12px", color: "var(--text-secondary)" }}>
                            <span style={{ "text-transform": "uppercase", background: "var(--bg-header)", padding: "2px 4px", "border-radius": "2px", "font-size": "10px" }}>
                                {activeItem()?.filename.split('.').pop()}
                            </span>
                        </div>
                    </div>
                     <div>
                        <span style={{ display: "block", "font-size": "10px", color: "var(--text-muted)", "margin-bottom": "2px" }}>CREATED</span>
                         <div style={{ display: "flex", "align-items": "center", gap: "6px", "font-size": "12px", color: "var(--text-secondary)" }}>
                            <Calendar size={12} />
                            <span>-</span>
                        </div>
                    </div>
                    <div>
                        <span style={{ display: "block", "font-size": "10px", color: "var(--text-muted)", "margin-bottom": "2px" }}>ID</span>
                         <div style={{ display: "flex", "align-items": "center", gap: "6px", "font-size": "12px", "font-family": "var(--font-mono)", color: "var(--text-muted)" }}>
                            <Hash size={12} />
                            {activeItem()?.id}
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    </div>
  );
};
