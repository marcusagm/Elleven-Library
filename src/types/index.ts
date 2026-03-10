/**
 * Represents an image item entity with metadata.
 */
export interface AssetItem {
    /**
     * Unique database ID
     *
     * @type {string}
     */
    id: string;

    /**
     * Absolute filesystem path
     *
     * @type {string}
     */
    path: string;

    /**
     * File name
     *
     * @type {string}
     */
    filename: string;

    /**
     * Width in pixels
     *
     * @type {number | null}
     */
    width: number | null;

    /**
     * Height in pixels
     *
     * @type {number | null}
     */
    height: number | null;

    /**
     * Path to extracted standard thumbnail
     *
     * @type {string | null}
     */
    thumbnail_path: string | null;

    /**
     * Asset user rating (0-5)
     *
     * @type {number}
     */
    rating: number;

    /**
     * Additional attached comments
     *
     * @type {string | null}
     */
    notes: string | null;

    /**
     * File size in bytes
     *
     * @type {number}
     */
    size: number;

    /**
     * Extracted format name
     *
     * @type {string}
     */
    format: string;

    /**
     * Categorized media type like Image, Video, etc
     *
     * @type {string}
     */
    media_type: string;

    /**
     * Creation timestamp
     *
     * @type {string}
     */
    created_at: string;

    /**
     * Last File modification timestamp
     *
     * @type {string}
     */
    modified_at: string;

    /**
     * Timestamp when added to library DB
     *
     * @type {string}
     */
    added_at: string;

    /**
     * Associated folder primary key
     *
     * @type {number}
     */
    folder_id: number;

    /**
     * Most prominent extracted color hex value (e.g., "#FF5733")
     *
     * @type {string | null}
     */
    dominant_color: string | null;
}

/**
 * Defines a supported file format and its corresponding metadata.
 */
export interface FileFormat {
    /**
     * Human readable name of the format
     *
     * @type {string}
     */
    name: string;

    /**
     * Accepted file extensions
     *
     * @type {string[]}
     */
    extensions: string[];

    /**
     * Linked MIME types mapping
     *
     * @type {string[]}
     */
    mimeTypes: string[];

    /**
     * Generic umbrella categorization for the underlying content
     *
     * @type {'Image' | 'Video' | 'Audio' | 'Project' | 'Archive' | 'Model3D' | 'Font' | 'Unknown'}
     */
    typeCategory:
        | 'Image'
        | 'Video'
        | 'Audio'
        | 'Project'
        | 'Archive'
        | 'Model3D'
        | 'Font'
        | 'Unknown';
}

// ==========================================
// V2 Backend DTOs
// ==========================================

export interface PageParams {
    page: number;
    pageSize: number;
}

export interface AssetFilter {
    family?: string;
    state?: string; // Based on AssetState enum
    searchQuery?: string;
    folderId?: string;
    tags?: string[];
    untagged?: boolean;
}

export type LogicalOperator = 'and' | 'or';

export interface SearchCriterion {
    id: string;
    key: string;
    operator: string;
    value: unknown;
}

export interface SearchGroup {
    id: string;
    logicalOperator: LogicalOperator;
    items: SearchItem[];
}

export type SearchItem = SearchGroup | SearchCriterion;

export interface SearchCriteria {
    id: string;
    rootGroup: SearchGroup;
}

export interface UpdateTagsPayload {
    assetId: string;
    tagsToAdd: string[];
    tagsToRemove: string[];
}

export interface CreateFolderPayload {
    parentId?: string;
    name: string;
    path: string;
}
