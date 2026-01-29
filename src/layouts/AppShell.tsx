import { Component, JSX } from "solid-js";
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "../components/ui/Resizable";
import "../styles/global.css";
import "./app-shell.css";

// This layout implements the 3-pane Grid structure with resizable areas
// [ Header ]
// [ Sidebar | Content | Inspector ]
// [ Statusbar ]

interface AppShellProps {
  children: JSX.Element;
  header?: JSX.Element;
  sidebar?: JSX.Element;
  inspector?: JSX.Element;
  statusbar?: JSX.Element;
}

export const AppShell: Component<AppShellProps> = (props) => {
  const STORAGE_KEY = "app-shell-layout";
  
  // Get persisted sizes or use defaults
  const getPersistedLayout = () => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? JSON.parse(saved) : null;
    } catch {
      return null;
    }
  };

  const layout = getPersistedLayout();
  const sidebarSize = layout?.[0] ?? 18;
  const contentSize = layout?.[1] ?? 62;
  const inspectorSize = layout?.[2] ?? 20;

  const handleLayoutChange = (sizes: number[]) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sizes));
  };

  return (
    <div class="app-shell">
      {/* Header Area */}
      <header class="shell-header">
        {props.header}
      </header>

      {/* Main 3-Pane Area - Resizable */}
      <ResizablePanelGroup 
        direction="horizontal" 
        class="shell-body"
        onLayout={handleLayoutChange}
      >
        {/* Left Sidebar */}
        <ResizablePanel 
          id="shell-sidebar" 
          defaultSize={sidebarSize} 
          minSize={12} 
          maxSize={35} 
          class="shell-sidebar"
        >
          {props.sidebar}
        </ResizablePanel>

        <ResizableHandle />

        {/* Central Viewport */}
        <ResizablePanel 
          id="shell-content" 
          defaultSize={contentSize} 
          minSize={30} 
          class="shell-content"
        >
          {props.children}
        </ResizablePanel>

        <ResizableHandle />

        {/* Right Inspector */}
        <ResizablePanel 
          id="shell-inspector" 
          defaultSize={inspectorSize} 
          minSize={15} 
          maxSize={40} 
          class="shell-inspector"
        >
          {props.inspector}
        </ResizablePanel>
      </ResizablePanelGroup>

      {/* Footer / Statusbar */}
      <footer class="shell-footer">
        {props.statusbar}
      </footer>
    </div>
  );
};
