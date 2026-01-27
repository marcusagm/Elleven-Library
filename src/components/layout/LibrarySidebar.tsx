import { Component } from "solid-js";
import { LibrarySidebarPanel } from "../features/library/LibrarySidebarPanel";
import { FoldersSidebarPanel } from "../features/library/FoldersSidebarPanel";
import { TagTreeSidebarPanel } from "../features/tags/TagTreeSidebarPanel";

export const LibrarySidebar: Component = () => {
    return (
        <aside class="library-sidebar" style={{ 
            height: "100%", 
            display: "flex", 
            "flex-direction": "column", 
            overflow: "hidden" 
        }}>
            <LibrarySidebarPanel />
            <FoldersSidebarPanel />
            <TagTreeSidebarPanel />
        </aside>
    );
};
