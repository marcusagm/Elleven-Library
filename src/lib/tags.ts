import { invoke } from '@tauri-apps/api/core';
import { type AssetItem } from '../types';

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
    createTag: async (
        name: string,
        parent_id?: number | null,
        color?: string | null
    ): Promise<number> => {
        return await invoke('create_tag', { name, parentId: parent_id, color });
    },

    updateTag: async (
        id: number,
        name?: string | null,
        color?: string | null,
        parent_id?: number | null,
        order_index?: number | null
    ): Promise<void> => {
        return await invoke('update_tag', {
            id,
            name,
            color,
            parentId: parent_id,
            orderIndex: order_index
        });
    },

    deleteTag: async (id: number): Promise<void> => {
        return await invoke('delete_tag', { id });
    },

    getAllTags: async (): Promise<Tag[]> => {
        return await invoke('get_all_tags');
    },

    getLibraryStats: async (): Promise<LibraryStats> => {
        return await invoke('get_library_stats');
    },

    addTagsToAssetsBatch: async (assetIds: number[], tagIds: number[]): Promise<void> => {
        return await invoke('add_tags_to_assets_batch', { assetIds: assetIds, tagIds });
    },

    getTagsForAsset: async (assetId: number): Promise<Tag[]> => {
        return await invoke('get_tags_for_asset', { assetId: assetId });
    },

    removeTagFromAsset: async (assetId: number, tagId: number): Promise<void> => {
        return await invoke('remove_tag_from_asset', { assetId: assetId, tagId });
    },

    getAssetsFiltered: async (
        limit: number,
        offset: number,
        tagIds: number[],
        matchAll: boolean = true,
        untagged?: boolean,
        folderId?: number,
        recursive: boolean = false,
        sort_by?: string,
        sort_order?: string,
        advanced_query?: string,
        search_query?: string
    ): Promise<AssetItem[]> => {
        return await invoke('get_assets_filtered', {
            limit,
            offset,
            tagIds,
            matchAll,
            untagged,
            folderId,
            recursive,
            sortBy: sort_by,
            sortOrder: sort_order,
            advancedQuery: advanced_query,
            searchQuery: search_query
        });
    },

    getAssetsFilteredCount: async (
        tagIds: number[],
        matchAll: boolean = true,
        untagged?: boolean,
        folderId?: number,
        recursive: boolean = false,
        advanced_query?: string,
        search_query?: string
    ): Promise<number> => {
        return await invoke('get_asset_count_filtered', {
            tagIds,
            matchAll,
            untagged,
            folderId,
            recursive,
            advancedQuery: advanced_query,
            searchQuery: search_query
        });
    },

    updateAssetRating: async (id: number, rating: number): Promise<void> => {
        return await invoke('update_asset_rating', { id, rating });
    },

    updateAssetNotes: async (id: number, notes: string): Promise<void> => {
        return await invoke('update_asset_notes', { id, notes });
    },

    removeTagsFromAssetsBatch: async (assetIds: number[], tagIds: number[]): Promise<void> => {
        return await invoke('remove_tags_from_assets_batch', { assetIds: assetIds, tagIds });
    },

    replaceTagsForAssetsBatch: async (assetIds: number[], tagIds: number[]): Promise<void> => {
        return await invoke('replace_tags_for_assets_batch', { assetIds: assetIds, tagIds });
    },

    getAssetExif: async (path: string): Promise<Record<string, string>> => {
        return await invoke('get_asset_exif', { path });
    }
};
