import { Component, Show } from "solid-js";
import { useLibrary, useSelection } from "../../core/hooks";
import "./global-statusbar.css";

export const GlobalStatusbar: Component = () => {
    const lib = useLibrary();
    const selection = useSelection();

  return (
    <div class="global-statusbar">
        <div class="statusbar-section">
            <span>{lib.items.length} Items</span>
            <Show when={selection.selectedIds.length > 0}>
                <span class="statusbar-selected">{selection.selectedIds.length} Selected</span>
            </Show>
        </div>
        <div class="statusbar-section">
             {/* Future: Sync Status, Cloud Icon */}
            <span>100%</span>
        </div>
    </div>
  );
};
