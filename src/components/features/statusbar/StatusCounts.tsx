import { Button } from '../../ui';
import { Component, Show } from 'solid-js';
import { useLibrary, useSelection } from '../../../core/hooks';
import { X } from 'lucide-solid';

export const StatusCounts: Component = () => {
    const lib = useLibrary();
    const selection = useSelection(); // Using existing hook

    const totalLoaded = () => lib.loadedCount();
    const totalFiltered = () => lib.totalItems;

    return (
        <div class="statusbar-section">
            <span title="Total items currently loaded in view">{totalLoaded()} Loaded</span>

            {/* Divider */}
            <span class="statusbar-divider" />

            <span title="Total items matching current filter (from backend)">
                {totalFiltered()} Total
            </span>

            <Show when={selection.selectedCount() > 0}>
                <span class="statusbar-divider" />
                <span class="statusbar-selected">{selection.selectedCount()} Selected</span>
                <Button
                    variant="ghost"
                    size="icon-xs"
                    class="status-btn"
                    title="Clear selection (Esc)"
                    onClick={e => {
                        e.stopPropagation();
                        selection.clear();
                    }}
                >
                    <X size={12} />
                </Button>
            </Show>
        </div>
    );
};
