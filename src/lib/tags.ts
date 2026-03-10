import { invokeCommand as invoke } from './api';
import {
    type AssetItem,
    type SearchCriteria,
    type AssetFilter,
    type UpdateTagsPayload
} from '../types';

export interface Tag {
    id: number;
    name: string;
    parent_id: number | null;
    color: string | null;
    order_index: number;
}

export interface LibraryStats {
    total_assets: number;
    untagged_assets: number;
    tag_counts: { tag_id: number; count: number }[];
    folder_counts: { folder_id: number; count: number }[];
    folder_counts_recursive: { folder_id: number; count: number }[];
}

export const tagService = {
    getAllTags: async (): Promise<Tag[]> => {
        return await invoke('list_tags');
    },

    getLibraryStats: async (): Promise<LibraryStats> => {
        return await invoke('get_library_stats'); // Assuming V2 keeps this unchanged
    },

    getTagsForAsset: async (assetId: string): Promise<Tag[]> => {
        return await invoke('get_tags_for_asset', { assetId: assetId }); // TBD V2 equivalent
    },

    searchAssets: async (
        criteria: SearchCriteria,
        page: number = 1,
        pageSize: number = 30
    ): Promise<AssetItem[]> => {
        return await invoke('search_assets', {
            criteria,
            page: { page, pageSize }
        });
    },

    getAssets: async (
        filter: AssetFilter,
        page: number = 1,
        pageSize: number = 30
    ): Promise<AssetItem[]> => {
        return await invoke('get_assets', {
            filter,
            page: { page, pageSize }
        });
    },

    updateAssetTags: async (payload: UpdateTagsPayload): Promise<void> => {
        return await invoke('update_asset_tags', { payload });
    },

    updateAssetRating: async (id: string, rating: number): Promise<void> => {
        return await invoke('update_asset_rating', { id, rating });
    },

    updateAssetNotes: async (id: string, notes: string): Promise<void> => {
        return await invoke('update_asset_notes', { id, notes });
    },

    createTag: async (
        name: string,
        parentId?: number | null,
        color?: string | null
    ): Promise<number> => {
        return await invoke('create_tag', { name, parentId, color });
    },

    updateTag: async (
        id: number,
        name?: string | null,
        color?: string | null,
        parentId?: number | null,
        orderIndex?: number | null
    ): Promise<void> => {
        return await invoke('update_tag', { id, name, color, parentId, orderIndex });
    },

    deleteTag: async (id: number): Promise<void> => {
        return await invoke('delete_tag', { id });
    },

    addTagsToAssetsBatch: async (assetIds: string[], tagIds: number[]): Promise<void> => {
        const tagsToAdd = tagIds.map(String);
        await Promise.all(
            assetIds.map(id =>
                tagService.updateAssetTags({
                    assetId: String(id),
                    tagsToAdd,
                    tagsToRemove: []
                })
            )
        );
    },

    removeTagsFromAssetsBatch: async (assetIds: string[], tagIds: number[]): Promise<void> => {
        const tagsToRemove = tagIds.map(String);
        await Promise.all(
            assetIds.map(id =>
                tagService.updateAssetTags({
                    assetId: String(id),
                    tagsToAdd: [],
                    tagsToRemove
                })
            )
        );
    },

    replaceTagsForAssetsBatch: async (assetIds: string[], tagIds: number[]): Promise<void> => {
        // V2 updateAssetTags does not inherently "replace", it just adds/removes. A true replace would need
        // backend support or explicit removing all then adding the new ones. For now, adapting to API.
        // Assuming we need a backend change or this is a destructive operation.
        return await invoke('replace_asset_tags', { assetIds, tagIds }); // Placeholder for true "replace" handling if implemented, or we map it to updateAssetTags.
    },

    getAssetExif: async (path: string): Promise<Record<string, string>> => {
        return await invoke('get_asset_exif', { path });
    }
};
