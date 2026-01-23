-- Elleven Library Schema

CREATE TABLE IF NOT EXISTS locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    location_id INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    size INTEGER,
    hash TEXT,
    created_at DATETIME NOT NULL,
    modified_at DATETIME NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    parent_id INTEGER,
    color TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS image_tags (
    image_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (image_id, tag_id),
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_images_path ON images(path);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
