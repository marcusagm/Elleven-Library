import { Component, JSX, Show } from "solid-js";
import "./sidebar-panel.css";

interface SidebarPanelProps {
  title: string;
  children: JSX.Element;
  actions?: JSX.Element;
  footer?: JSX.Element;
  class?: string;
  contentClass?: string;
  style?: JSX.CSSProperties;
  onDragOver?: (e: any) => void;
  onDragLeave?: (e: any) => void;
  onDrop?: (e: any) => void;
}

export const SidebarPanel: Component<SidebarPanelProps> = (props) => {
  return (
    <div class={`sidebar-panel ${props.class || ""}`} style={props.style}>
      <div class="sidebar-panel-header">
        <span class="sidebar-panel-title">{props.title}</span>
        <Show when={props.actions}>
          <div class="sidebar-panel-actions">
            {props.actions}
          </div>
        </Show>
      </div>
      <div 
        class={`sidebar-panel-content ${props.contentClass || ""}`}
        onDragOver={props.onDragOver}
        onDragLeave={props.onDragLeave}
        onDrop={props.onDrop}
      >
        {props.children}
      </div>
      <Show when={props.footer}>
        <div class="sidebar-panel-footer">
          {props.footer}
        </div>
      </Show>
    </div>
  );
};
