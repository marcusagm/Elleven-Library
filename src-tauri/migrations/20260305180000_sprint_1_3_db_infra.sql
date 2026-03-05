-- Sprint 1.3: Data Model Base (CQRS Infra)
-- Creating V2 tables for the new architecture, ensuring parallel existence with V1.

-- Main Assets Table (V2)
CREATE TABLE IF NOT EXISTS v2_assets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT UNIQUE NOT NULL,
    state TEXT NOT NULL,
    format_type TEXT NOT NULL,
    family TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- Metadata Envelope (V2)
CREATE TABLE IF NOT EXISTS v2_asset_metadata_envelope (
    asset_id TEXT PRIMARY KEY,
    width INTEGER,
    height INTEGER,
    duration_secs REAL,
    dominant_colors JSON,
    technical_payload JSON,
    semantic_payload JSON,
    FOREIGN KEY (asset_id) REFERENCES v2_assets(id) ON DELETE CASCADE
);

-- Thumbnails Registry (V2)
CREATE TABLE IF NOT EXISTS v2_asset_thumbnails_registry (
    asset_id TEXT PRIMARY KEY,
    has_small BOOLEAN DEFAULT 0,
    has_medium BOOLEAN DEFAULT 0,
    has_large BOOLEAN DEFAULT 0,
    extracted_at TIMESTAMP,
    format_provider TEXT,
    FOREIGN KEY (asset_id) REFERENCES v2_assets(id) ON DELETE CASCADE
);

-- Operations Log (CQRS Audit)
CREATE TABLE IF NOT EXISTS v2_asset_operations_log (
    id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    payload JSON NOT NULL,
    status TEXT NOT NULL,
    error_note TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indices for V2
CREATE INDEX IF NOT EXISTS idx_v2_assets_path ON v2_assets(path);
CREATE INDEX IF NOT EXISTS idx_v2_assets_state ON v2_assets(state);
