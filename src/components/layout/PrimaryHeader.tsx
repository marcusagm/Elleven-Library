import { Component, Show } from "solid-js";
import { appActions, useAppStore } from "../../core/store/appStore";
import { Search, ChevronLeft, ChevronRight, Plus, Loader2 } from "lucide-solid";

export const PrimaryHeader: Component = () => {
  const { searchQuery, progress } = useAppStore();

  return (
    <div style={{ 
        padding: "0 16px", 
        height: "52px", 
        display: "flex", 
        "align-items": "center", 
        gap: "16px",
        background: "var(--bg-header)",
        "border-bottom": "1px solid var(--border-color)"
    }}>
       {/* Navigation */}
       <div style={{ display: "flex", gap: "8px" }}>
         <button class="icon-btn" disabled>
            <ChevronLeft size={16} />
         </button>
         <button class="icon-btn" disabled>
            <ChevronRight size={16} />
         </button>
       </div>

       {/* OmniSearch */}
       <div style={{ flex: 1, display: "flex", "justify-content": "center" }}>
           <div style={{ 
               width: "100%", 
               "max-width": "600px", 
               position: "relative",
               color: "var(--text-secondary)"
           }}>
               <Search size={14} style={{ position: "absolute", left: "12px", top: "50%", transform: "translateY(-50%)" }} />
               <input 
                 type="text" 
                 placeholder="Search references (Ctrl+K)" 
                 value={searchQuery()}
                 onInput={(e) => appActions.setSearch(e.currentTarget.value)}
                 style={{ 
                     width: "100%", 
                     "background-color": "var(--bg-color)",
                     "border": "1px solid var(--border-color)",
                     "border-radius": "var(--radius-md)",
                     "padding": "8px 12px 8px 36px",
                     "font-size": "13px",
                     "color": "var(--text-primary)",
                     "outline": "none",
                     "transition": "border-color 0.2s"
                 }}
                 onFocus={(e) => e.currentTarget.style.borderColor = "var(--border-active)"}
                 onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-color)"}
                />
           </div>
       </div>

       {/* Actions / Status */}
       <div style={{ display: "flex", "align-items": "center", gap: "16px", "font-size": "11px" }}>
            <Show when={progress()}>
                <div style={{ display: "flex", "align-items": "center", gap: "8px", color: "var(--text-secondary)" }}>
                    <Loader2 size={12} class="spin" />
                    <span>Indexing {progress()?.processed} / {progress()?.total}</span>
                </div>
            </Show>
            <button 
                class="primary-btn" 
                style={{ 
                    "font-size": "12px", 
                    padding: "6px 12px", 
                    display: "flex", 
                    "align-items": "center", 
                    gap: "6px",
                    "border-radius": "var(--radius-sm)"
                }}>
                <Plus size={14} />
                <span>Add</span>
            </button>
       </div>
    </div>
  );
};
