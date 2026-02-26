/**
 * useAssetCardActions
 *
 * Centralizes all actions for asset cards, keeping the AssetCard component pure.
 * This hook should be used by viewport containers (VirtualMasonry, VirtualGridView)
 * to get callbacks to pass down to AssetCard components.
 */

import { useSelection } from './useSelection';
import { useViewport } from './useViewport';
import { libraryState } from '../store/libraryStore';
import { type ImageItem } from '../../types';

export interface AssetCardActions {
    /** Toggle selection for an item, optionally with multi-select or range-select */
    handleSelect: (id: number, modifiers: { multi: boolean; shift: boolean }) => void;
    /** Open item in detail/preview view */
    handleOpen: (id: number) => void;
    /** Check if an item is currently selected */
    isSelected: (id: number) => boolean;
    /** Get all currently selected IDs (for DnD) */
    getSelectedIds: () => (number | string)[];
}

export function useAssetCardActions(): AssetCardActions {
    const selection = useSelection();
    const viewport = useViewport();

    return {
        handleSelect: (id: number, modifiers: { multi: boolean; shift: boolean }) => {
            if (modifiers.shift) {
                const allIds = libraryState.items.map((item: ImageItem) => item.id);
                selection.selectRange(id, allIds);
            } else {
                selection.toggle(id, modifiers.multi);
            }
        },

        handleOpen: (id: number) => {
            viewport.openItem(id.toString());
        },

        isSelected: (id: number) => {
            return selection.selectedIds.includes(id);
        },

        getSelectedIds: () => {
            return selection.selectedIds;
        }
    };
}
