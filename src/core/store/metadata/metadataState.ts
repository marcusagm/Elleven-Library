import { createStore } from 'solid-js/store';
import { Tag } from '../../../lib/tags';

export interface FolderNode {
    id: string;
    path: string;
    name: string;
    parent_id: string | null;
    is_root: boolean;
}

export interface SmartFolder {
    id: string;
    name: string;
    query_json: string;
    created_at: string;
}

export interface MetadataState {
    tags: Tag[];
    locations: FolderNode[];
    smartFolders: SmartFolder[];
    libraryStats: {
        total_assets: number;
        untagged_assets: number;
        tag_counts: Map<string, number>;
        folder_counts: Map<string, number>;
        folder_counts_recursive: Map<string, number>;
    };
    tagUpdateVersion: number;
}

export const [metadataState, setMetadataState] = createStore<MetadataState>({
    tags: [],
    locations: [],
    smartFolders: [],
    libraryStats: {
        total_assets: 0,
        untagged_assets: 0,
        tag_counts: new Map(),
        folder_counts: new Map(),
        folder_counts_recursive: new Map()
    },
    tagUpdateVersion: 0
});
