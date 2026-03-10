-- Sprint 6.3: Authoritative Final Schema Normalization
-- Consolidates cleanup, bridge-fallback, and data migration.

PRAGMA foreign_keys = OFF;

-- 1. DROP ALL LEGACY V1 TABLES AND TRIGGERS (Idempotent)
DROP TABLE IF EXISTS assets_fts;
DROP TRIGGER IF EXISTS assets_ai;
DROP TRIGGER IF EXISTS assets_ad;
DROP TRIGGER IF EXISTS assets_au;
DROP TABLE IF EXISTS asset_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS assets;
DROP TABLE IF EXISTS folders;
DROP TABLE IF EXISTS smart_folders;
DROP TABLE IF EXISTS app_settings;
DROP TABLE IF EXISTS asset_colors;

-- 2. DROP UNUSED V2 INFRA (Idempotent)
DROP TABLE IF EXISTS v2_asset_thumbnails_registry;
DROP TABLE IF EXISTS asset_thumbnails_registry;

-- 3. ENSURE TRANSITIONAL BRIDGES EXIST (Dummy empty tables if V2 didn't exist yet)
-- This guarantees the SELECT statements later won't fail.

CREATE TABLE IF NOT EXISTS v2_folders (
    id TEXT PRIMARY KEY, parent_id TEXT, name TEXT, path TEXT, created_at TIMESTAMP, updated_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS v2_assets (
    id TEXT PRIMARY KEY, name TEXT, path TEXT, state TEXT, format_type TEXT, family TEXT,
    file_size INTEGER, created_at TIMESTAMP, updated_at TIMESTAMP, folder_id TEXT,
    thumbnail_path TEXT, dominant_color TEXT
);

CREATE TABLE IF NOT EXISTS v2_tags (
    id TEXT PRIMARY KEY, name TEXT, color TEXT, parent_id TEXT, created_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS v2_asset_tags (
    asset_id TEXT, tag_id TEXT, PRIMARY KEY(asset_id, tag_id)
);

CREATE TABLE IF NOT EXISTS v2_asset_metadata_envelope (
    asset_id TEXT PRIMARY KEY, width INTEGER, height INTEGER, duration_secs REAL,
    technical_payload TEXT, semantic_payload TEXT, dominant_colors TEXT
);

CREATE TABLE IF NOT EXISTS v2_asset_operations_log (
    id TEXT PRIMARY KEY, operation_type TEXT, asset_id TEXT, payload TEXT, status TEXT,
    error_note TEXT, created_at TIMESTAMP
);

-- 4. CREATE FINAL V2 CLEAN TABLES

CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    parent_id TEXT,
    name TEXT NOT NULL,
    path TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);

CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT UNIQUE NOT NULL,
    state TEXT NOT NULL,
    format_type TEXT NOT NULL,
    family TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    folder_id TEXT,
    thumbnail_path TEXT,
    dominant_color TEXT,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_assets_path ON assets(path);
CREATE INDEX IF NOT EXISTS idx_assets_state ON assets(state);
CREATE INDEX IF NOT EXISTS idx_assets_folder ON assets(folder_id);
CREATE INDEX IF NOT EXISTS idx_assets_thumbnail_path ON assets(thumbnail_path);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    color TEXT,
    parent_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS asset_tags (
    asset_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (asset_id, tag_id),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_asset_tags_asset ON asset_tags(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_tags_tag ON asset_tags(tag_id);

CREATE TABLE IF NOT EXISTS asset_metadata_envelope (
    asset_id TEXT PRIMARY KEY,
    width INTEGER,
    height INTEGER,
    duration_secs REAL,
    technical_payload TEXT,
    semantic_payload TEXT,
    dominant_colors TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS asset_operations_log (
    id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    status TEXT NOT NULL,
    error_note TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS asset_colors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id TEXT NOT NULL,
    hex_color TEXT NOT NULL,
    lab_lightness REAL NOT NULL,
    lab_green_red REAL NOT NULL,
    lab_blue_yellow REAL NOT NULL,
    percentage REAL NOT NULL,
    rank INTEGER NOT NULL,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_asset_colors_asset ON asset_colors(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_colors_lab ON asset_colors(lab_lightness, lab_green_red, lab_blue_yellow);

-- 5. PERFORM DATA MIGRATION from v2_ (Normalized Move)
-- Using INSERT OR IGNORE and explicit columns.

INSERT OR IGNORE INTO folders (id, parent_id, name, path, created_at, updated_at)
SELECT id, parent_id, name, path, created_at, updated_at FROM v2_folders;

INSERT OR IGNORE INTO assets (id, name, path, state, format_type, family, file_size, created_at, updated_at, folder_id, thumbnail_path, dominant_color)
SELECT id, name, path, state, format_type, family, file_size, created_at, updated_at, folder_id, thumbnail_path, dominant_color FROM v2_assets;

INSERT OR IGNORE INTO tags (id, name, color, parent_id, created_at)
SELECT id, name, color, parent_id, created_at FROM v2_tags;

INSERT OR IGNORE INTO asset_tags (asset_id, tag_id)
SELECT asset_id, tag_id FROM v2_asset_tags;

INSERT OR IGNORE INTO asset_metadata_envelope (asset_id, width, height, duration_secs, technical_payload, semantic_payload, dominant_colors)
SELECT asset_id, width, height, duration_secs, technical_payload, semantic_payload, dominant_colors FROM v2_asset_metadata_envelope;

INSERT OR IGNORE INTO asset_operations_log (id, operation_type, asset_id, payload, status, error_note, created_at)
SELECT id, operation_type, asset_id, payload, status, error_note, created_at FROM v2_asset_operations_log;

-- 6. FINAL CLEANUP OF V2 TRANSITIONAL INFRA
DROP TABLE IF EXISTS v2_folders;
DROP TABLE IF EXISTS v2_assets;
DROP TABLE IF EXISTS v2_tags;
DROP TABLE IF EXISTS v2_asset_tags;
DROP TABLE IF EXISTS v2_asset_metadata_envelope;
DROP TABLE IF EXISTS v2_asset_operations_log;

PRAGMA foreign_keys = ON;
