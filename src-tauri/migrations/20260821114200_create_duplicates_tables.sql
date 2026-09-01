-- Migration to create the duplicates detection subsystem tables

-- 1. duplicate_fingerprints
CREATE TABLE duplicate_fingerprints (
    asset_id TEXT PRIMARY KEY,
    content_hash TEXT,
    perceptual_hash TEXT,
    block_hash TEXT,
    thumb_hash TEXT,
    width INTEGER,
    height INTEGER,
    file_size INTEGER,
    mime_type TEXT,
    format_family TEXT,
    color_profile TEXT,
    orientation INTEGER,
    fingerprint_version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- 2. duplicate_rule_sets
CREATE TABLE duplicate_rule_sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    consider_exact_match INTEGER NOT NULL DEFAULT 1,
    consider_visual_match INTEGER NOT NULL DEFAULT 1,
    consider_crop_match INTEGER NOT NULL DEFAULT 0,
    ignore_resolution_difference INTEGER NOT NULL DEFAULT 1,
    ignore_recompression INTEGER NOT NULL DEFAULT 1,
    allow_rotation INTEGER NOT NULL DEFAULT 1,
    allow_mirroring INTEGER NOT NULL DEFAULT 0,
    min_score REAL NOT NULL DEFAULT 0.85,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 3. duplicate_groups
CREATE TABLE duplicate_groups (
    id TEXT PRIMARY KEY,
    rule_set_id TEXT NOT NULL,
    group_type TEXT NOT NULL, -- exact | near | derived
    canonical_asset_id TEXT,
    confidence REAL NOT NULL,
    status TEXT NOT NULL, -- open | reviewed | ignored | resolved
    candidate_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(rule_set_id) REFERENCES duplicate_rule_sets(id)
);

-- 4. duplicate_candidates
CREATE TABLE duplicate_candidates (
    group_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    score REAL NOT NULL,
    reasons TEXT NOT NULL, -- JSON array
    is_selected INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(group_id, asset_id),
    FOREIGN KEY(group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- 5. duplicate_resolutions
CREATE TABLE duplicate_resolutions (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    action TEXT NOT NULL, -- keep_one | delete_selected | merge_metadata | ignore_group | custom
    selected_asset_id TEXT,
    payload TEXT, -- JSON com detalhes da decisão
    resolved_by TEXT,
    resolved_at TEXT NOT NULL,
    FOREIGN KEY(group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_duplicate_fingerprints_content_hash ON duplicate_fingerprints(content_hash);
CREATE INDEX idx_duplicate_fingerprints_phash ON duplicate_fingerprints(perceptual_hash);
CREATE INDEX idx_duplicate_groups_status ON duplicate_groups(status);
CREATE INDEX idx_duplicate_candidates_asset_id ON duplicate_candidates(asset_id);
