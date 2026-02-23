import { invoke } from '@tauri-apps/api/core';
import { type ImageItem } from '../types';

// We primarily use the Rust backend for DB operations now.
// This file wraps those invocations or provides legacy support where needed.

interface FolderNode {
    id: number;
    path: string;
    name: string;
    parent_id: number | null;
    is_root: boolean;
}

export async function initDb() {
    // Database management is handled by the Backend (Rust).
    // No-op or perform specific frontend-only inits if needed
}

export async function addLocation(path: string) {
    return await invoke('add_location', { path });
}

export async function getLocations() {
    return await invoke<FolderNode[]>('get_locations');
}

export async function getImages(
    limit: number = 100,
    offset: number = 0,
    sortBy?: string,
    sortOrder?: string
) {
    // Use the backend command which now handles the unified logic
    return await invoke<ImageItem[]>('get_images_filtered', {
        limit,
        offset,
        tagIds: [],
        matchAll: true,
        untagged: false,
        folderId: null,
        recursive: true,
        sortBy,
        sortOrder
    });
}
