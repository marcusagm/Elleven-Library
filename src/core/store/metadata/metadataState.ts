import { createStore } from 'solid-js/store';
import { Tag } from '../../../lib/tags';

export interface FolderNode {
    id: number;
    path: string;
    name: string;
    parent_id: number | null;
    is_root: boolean;
}

export interface SmartFolder {
    id: number;
    name: string;
    query_json: string;
    created_at: string;
}

export interface MetadataState {
    tags: Tag[];
    locations: FolderNode[];
    smartFolders: SmartFolder[];
    libraryStats: {
        total_images: number;
        untagged_images: number;
        tag_counts: Map<number, number>;
        folder_counts: Map<number, number>;
        folder_counts_recursive: Map<number, number>;
    };
    tagUpdateVersion: number;
}

export const [metadataState, setMetadataState] = createStore<MetadataState>({
    tags: [],
    locations: [],
    smartFolders: [],
    libraryStats: {
        total_images: 0,
        untagged_images: 0,
        tag_counts: new Map(),
        folder_counts: new Map(),
        folder_counts_recursive: new Map()
    },
    tagUpdateVersion: 0
});
