import { Component, Show } from "solid-js";
import { appActions, useAppStore } from "../../core/store/appStore";
import { Search, ChevronLeft, ChevronRight, Plus, LoaderCircle } from "lucide-solid";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";

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
         <Button variant="ghost" size="icon" disabled>
            <ChevronLeft size={16} />
         </Button>
         <Button variant="ghost" size="icon" disabled>
            <ChevronRight size={16} />
         </Button>
       </div>

       {/* OmniSearch */}
       <div style={{ flex: 1, display: "flex", "justify-content": "center" }}>
           <div style={{ width: "100%", "max-width": "600px" }}>
               <Input 
                 placeholder="Search references (Ctrl+K)" 
                 value={searchQuery()}
                 onInput={(e) => appActions.setSearch(e.currentTarget.value)}
                 leftIcon={<Search size={14} />}
               />
           </div>
       </div>

       {/* Actions / Status */}
       <div style={{ display: "flex", "align-items": "center", gap: "16px", "font-size": "11px" }}>
            <Show when={progress()}>
                <div style={{ display: "flex", "align-items": "center", gap: "8px", color: "var(--text-secondary)" }}>
                    <LoaderCircle size={12} class="spin" />
                    <span>Indexing {progress()?.processed} / {progress()?.total}</span>
                </div>
            </Show>
            <Button variant="primary" size="sm">
                <Plus size={14} />
                <span>Add</span>
            </Button>
       </div>
    </div>
  );
};
