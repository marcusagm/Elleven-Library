-- Sprint 2.4: Taxonomy, Metadata and Folders
-- Implementing recursive folders and N:N tags.

-- 1. Folders Table (Self-Referential)
CREATE TABLE IF NOT EXISTS v2_folders (
    id TEXT PRIMARY KEY,
    parent_id TEXT,
    name TEXT NOT NULL,
    path TEXT UNIQUE NOT NULL, -- Absolute path for logic / sync
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES v2_folders(id) ON DELETE CASCADE
);

-- 2. Tags Table
CREATE TABLE IF NOT EXISTS v2_tags (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT, -- Hex color
    parent_id TEXT, -- For nested tags
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES v2_tags(id) ON DELETE SET NULL
);

-- 3. Asset-Tags Pivot Table
CREATE TABLE IF NOT EXISTS v2_asset_tags (
    asset_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (asset_id, tag_id),
    FOREIGN KEY (asset_id) REFERENCES v2_assets(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES v2_tags(id) ON DELETE CASCADE
);

-- 4. Alter Assets to link with Folders
ALTER TABLE v2_assets ADD COLUMN folder_id TEXT REFERENCES v2_folders(id) ON DELETE SET NULL;

-- 5. Indices for performance
CREATE INDEX IF NOT EXISTS idx_v2_folders_parent ON v2_folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_v2_assets_folder ON v2_assets(folder_id);
CREATE INDEX IF NOT EXISTS idx_v2_asset_tags_asset ON v2_asset_tags(asset_id);
CREATE INDEX IF NOT EXISTS idx_v2_asset_tags_tag ON v2_asset_tags(tag_id);
