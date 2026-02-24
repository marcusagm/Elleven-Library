/**
 * Represents an image item entity with metadata.
 */
export interface ImageItem {
    /** Unique database ID */
    id: number;
    /** Absolute filesystem path */
    path: string;
    /** File name */
    filename: string;
    /** Width in pixels */
    width: number | null;
    /** Height in pixels */
    height: number | null;
    /** Path to extracted standard thumbnail */
    thumbnail_path: string | null;
    /** Asset user rating (0-5) */
    rating: number;
    /** Additional attached comments */
    notes: string | null;
    /** File size in bytes */
    size: number;
    /** Extracted format name */
    format: string;
    /** Creation timestamp */
    created_at: string;
    /** Last File modification timestamp */
    modified_at: string;
    /** Timestamp when added to library DB */
    added_at: string;
    /** Associated folder primary key */
    folder_id: number;
}

/**
 * Defines a supported file format and its corresponding metadata.
 */
export interface FileFormat {
    /** Human readable name of the format */
    name: string;
    /** Accepted file extensions */
    extensions: string[];
    /** Linked MIME types mapping */
    mimeTypes: string[];
    /** Generic umbrella categorization for the underlying content */
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
