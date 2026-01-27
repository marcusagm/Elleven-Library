import { Component, For } from "solid-js";
import { Folder, Plus } from "lucide-solid";
import { useAppStore, appActions } from "../../../core/store/appStore";
import { CountBadge } from "../../ui/CountBadge";
import { Button } from "../../ui/Button";
import { SidebarPanel } from "../../ui/SidebarPanel";

export const FoldersSidebarPanel: Component = () => {
    const { state } = useAppStore();

    return (
        <SidebarPanel 
            title="Folders" 
            style={{ flex: "0 1 auto", "max-height": "40%" }}
            actions={
                <Button variant="ghost" size="icon-sm" title="Create Folder">
                    <Plus size={14} />
                </Button>
            }
        >
            <div style={{ "display": "flex", "flex-direction": "column", gap: "1px" }}>
                <For each={state.locations}>
                    {(loc) => (
                        <div 
                            class={`nav-item ${state.selectedLocationId === (loc as any).id ? 'active' : ''}`} 
                            title={loc.path}
                            onClick={() => appActions.selectLocation((loc as any).id)}
                        >
                            <Folder size={16} fill="var(--text-muted)" stroke="none" /> 
                            <span style={{ flex: 1, "white-space": "nowrap", "overflow": "hidden", "text-overflow": "ellipsis" }}>
                                {loc.name}
                            </span>
                            <CountBadge count={state.libraryStats.folder_counts.get((loc as any).id) || 0} variant="outline" />
                        </div>
                    )}
                </For>
                
                {state.locations.length === 0 && (
                    <div style={{ padding: "12px", "text-align": "center", color: "var(--text-muted)", "font-size": "11px", "font-style": "italic" }}>
                        No folders linked
                    </div>
                )}
            </div>
        </SidebarPanel>
    );
};
