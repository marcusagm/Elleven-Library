-- Elleven Library Schema

CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_root BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_id INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    size INTEGER,
    hash TEXT,
    thumbnail_path TEXT,
    format TEXT,
    media_type TEXT NOT NULL,
    rating INTEGER DEFAULT 0,
    notes TEXT,
    created_at DATETIME NOT NULL,
    modified_at DATETIME NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    thumbnail_attempts INTEGER DEFAULT 0,
    thumbnail_last_error TEXT,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    parent_id INTEGER,
    color TEXT,
    order_index INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS asset_tags (
    asset_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (asset_id, tag_id),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS smart_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    query_json TEXT NOT NULL, -- structured query object as JSON string
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_assets_path ON assets(path);
CREATE INDEX IF NOT EXISTS idx_assets_folder ON assets(folder_id);
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_folders_path ON folders(path);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

-- Performance Indices for Sorting
CREATE INDEX IF NOT EXISTS idx_assets_rating_created ON assets(rating DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_assets_modified ON assets(modified_at DESC);
CREATE INDEX IF NOT EXISTS idx_assets_size ON assets(size DESC);
CREATE INDEX IF NOT EXISTS idx_assets_created ON assets(created_at DESC);

-- FTS5 Virtual Table for Fast Text Search
-- Uses 'trigram' tokenizer for efficient substring matching (LIKE %query%)
-- content='assets' makes it an external content table (saves space)
CREATE VIRTUAL TABLE IF NOT EXISTS assets_fts USING fts5(
    filename,
    notes,
    content='assets',
    content_rowid='id',
    tokenize='trigram'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS assets_ai AFTER INSERT ON assets BEGIN
  INSERT INTO assets_fts(rowid, filename, notes) VALUES (new.id, new.filename, new.notes);
END;

CREATE TRIGGER IF NOT EXISTS assets_ad AFTER DELETE ON assets BEGIN
  INSERT INTO assets_fts(assets_fts, rowid, filename, notes) VALUES('delete', old.id, old.filename, old.notes);
END;

CREATE TRIGGER IF NOT EXISTS assets_au AFTER UPDATE ON assets BEGIN
  INSERT INTO assets_fts(assets_fts, rowid, filename, notes) VALUES('delete', old.id, old.filename, old.notes);
  INSERT INTO assets_fts(rowid, filename, notes) VALUES (new.id, new.filename, new.notes);
END;

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL, -- JSON Value
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

