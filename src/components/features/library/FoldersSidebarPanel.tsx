import { Component, For } from "solid-js";
import { Folder, Plus } from "lucide-solid";
import { useMetadata, useFilters } from "../../../core/hooks";
import { CountBadge } from "../../ui/CountBadge";
import { Button } from "../../ui/Button";
import { SidebarPanel } from "../../ui/SidebarPanel";

export const FoldersSidebarPanel: Component = () => {
    const metadata = useMetadata();
    const filters = useFilters();

    return (
        <SidebarPanel 
            title="Folders" 
            class="panel-limited"
            actions={
                <Button variant="ghost" size="icon-sm" title="Create Folder">
                    <Plus size={14} />
                </Button>
            }
        >
            <div class="sidebar-list">
                <For each={metadata.locations}>
                    {(loc) => (
                        <div 
                            class={`nav-item ${filters.selectedLocationId === (loc as any).id ? 'active' : ''}`} 
                            title={loc.path}
                            onClick={() => filters.setLocation((loc as any).id)}
                        >
                            <Folder size={16} fill="var(--text-muted)" stroke="none" /> 
                            <span class="truncate" style={{ flex: 1 }}>
                                {loc.name}
                            </span>
                            <CountBadge count={metadata.stats.folder_counts.get((loc as any).id) || 0} variant="outline" />
                        </div>
                    )}
                </For>
                
                {metadata.locations.length === 0 && (
                    <div class="sidebar-empty-state">
                        No folders linked
                    </div>
                )}
            </div>
        </SidebarPanel>
    );
};
