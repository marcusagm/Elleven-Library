import { type AssetItem } from '../../../types';

export interface BatchChangeAddedItem extends AssetItem {
    folder_id: number;
    old_folder_id?: number;
}

export interface BatchChangeRemovedItem {
    id: string;
    folder_id: number;
    tag_ids: number[];
}

export interface BatchChangePayload {
    added?: BatchChangeAddedItem[];
    removed?: BatchChangeRemovedItem[];
    updated?: BatchChangeAddedItem[];
    needs_refresh?: boolean;
}
