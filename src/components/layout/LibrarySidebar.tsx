import { Component } from 'solid-js';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '../ui/Resizable';
import { LibrarySidebarPanel } from '../features/library/LibrarySidebarPanel';
import { FolderTreeSidebarPanel } from '../features/library/FolderTreeSidebarPanel';
import { TagTreeSidebarPanel } from '../features/tags/TagTreeSidebarPanel';
import { SmartFoldersSidebarPanel } from '../features/search/SmartFoldersSidebarPanel';
import './library-sidebar.css';

/**
 * Sidebar component for the library area, featuring vertical resizable panels.
 * Manages the layout persistence and coordination of library sub-panels.
 *
 * @returns {JSX.Element} The rendered library sidebar.
 */
export const LibrarySidebar: Component = () => {
    /** Persistence key for the sidebar panel layout */
    const LAYOUT_STORAGE_KEY = 'sidebar-layout-v2';

    /**
     * Retrieves the persisted layout from local storage.
     *
     * @returns {number[] | null} An array of panel sizes or null if not found.
     */
    const getPersistedLayout = (): number[] | null => {
        try {
            const savedLayout = localStorage.getItem(LAYOUT_STORAGE_KEY);
            return savedLayout ? JSON.parse(savedLayout) : null;
        } catch (error) {
            console.warn('Failed to load library sidebar layout:', error);
            return null;
        }
    };

    const persistedLayout = getPersistedLayout();

    // Initial sizes for the vertical panels
    const libraryPanelSize = persistedLayout?.[0] ?? 15;
    const foldersPanelSize = persistedLayout?.[1] ?? 35;
    const tagsPanelSize = persistedLayout?.[2] ?? 30;
    const smartFoldersPanelSize = persistedLayout?.[3] ?? 20;

    /**
     * Handles changes to the panel sizes and persists them.
     *
     * @param {number[]} newPanelSizes - The updated sizes of all panels in the group.
     */
    const handleLayoutChange = (newPanelSizes: number[]) => {
        localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify(newPanelSizes));
    };

    return (
        <aside class="library-sidebar">
            <ResizablePanelGroup direction="vertical" onLayout={handleLayoutChange}>
                <ResizablePanel
                    id="sidebar-library"
                    defaultSize={libraryPanelSize}
                    minSize={10}
                    class="panel-lib"
                >
                    <LibrarySidebarPanel />
                </ResizablePanel>

                <ResizableHandle />

                <ResizablePanel
                    id="sidebar-folders"
                    defaultSize={foldersPanelSize}
                    minSize={15}
                    class="panel-folders"
                >
                    <FolderTreeSidebarPanel />
                </ResizablePanel>

                <ResizableHandle />

                <ResizablePanel
                    id="sidebar-tags"
                    defaultSize={tagsPanelSize}
                    minSize={15}
                    class="panel-tags"
                >
                    <TagTreeSidebarPanel />
                </ResizablePanel>

                <ResizableHandle />

                <ResizablePanel
                    id="sidebar-smart"
                    defaultSize={smartFoldersPanelSize}
                    minSize={10}
                    class="panel-smart"
                >
                    <SmartFoldersSidebarPanel />
                </ResizablePanel>
            </ResizablePanelGroup>
        </aside>
    );
};
