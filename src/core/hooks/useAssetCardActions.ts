/**
 * useAssetCardActions
 *
 * Centralizes all actions for asset cards, keeping the AssetCard component pure.
 * This hook should be used by viewport containers (VirtualMasonry, VirtualGridView)
 * to get callbacks to pass down to AssetCard components.
 */

import { useSelection } from './useSelection';
import { useViewport } from './useViewport';
import { libraryState } from '../store/library';
import { type AssetItem } from '../../types';

export interface AssetCardActions {
    /** Toggle selection for an item, optionally with multi-select or range-select */
    handleSelect: (itemId: string, modifiers: { multi: boolean; shift: boolean }) => void;
    /** Open item in detail/preview view */
    handleOpen: (itemId: string) => void;
    /** Check if an item is currently selected */
    isSelected: (itemId: string) => boolean;
    /** Get all currently selected IDs (for DnD) */
    getSelectedIds: () => string[];
}

export function useAssetCardActions(): AssetCardActions {
    const selection = useSelection();
    const viewport = useViewport();

    return {
        handleSelect: (itemId: string, modifiers: { multi: boolean; shift: boolean }) => {
            if (modifiers.shift) {
                const allIdentifiers = libraryState.items.map((item: AssetItem) => item.id);
                selection.selectRange(itemId, allIdentifiers);
            } else {
                selection.toggle(itemId, modifiers.multi);
            }
        },

        handleOpen: (itemId: string) => {
            viewport.openItem(itemId);
        },

        isSelected: (itemId: string) => {
            return selection.selectedIds.includes(itemId);
        },

        getSelectedIds: () => {
            return selection.selectedIds;
        }
    };
}
