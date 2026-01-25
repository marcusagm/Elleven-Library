import { Component, Show } from "solid-js";
import { useAppStore } from "../../core/store/appStore";

export const GlobalStatusbar: Component = () => {
    const { state } = useAppStore();

  return (
    <div style={{ 
        display: "flex", 
        width: "100%", 
        "justify-content": "space-between", 
        "padding": "0 12px",
        "font-size": "11px",
        "font-weight": "500",
        color: "var(--text-muted)",
        "letter-spacing": "0.3px"
    }}>
        <div style={{ display: "flex", gap: "16px" }}>
            <span>{state.items.length} Items</span>
            <Show when={state.selection.length > 0}>
                <span style={{ color: "var(--accent-color)" }}>{state.selection.length} Selected</span>
            </Show>
        </div>
        <div style={{ display: "flex", gap: "16px" }}>
             {/* Future: Sync Status, Cloud Icon */}
            <span>100%</span>
        </div>
    </div>
  );
};
