-- Sprint 3: Performance & Indexing Tuning for SQLite Sub-System

-- Optimize JOINs for Tags.
-- Since `asset_tags` has PRIMARY KEY (asset_id, tag_id), queries filtering by `tag_id`
-- do a full table scan without this reverse index.
CREATE INDEX IF NOT EXISTS idx_asset_tags_tag_id ON asset_tags(tag_id, asset_id);

-- Optimize ORDER BY filename which uses COLLATE NOCASE in queries.
CREATE INDEX IF NOT EXISTS idx_assets_filename_nocase ON assets(filename COLLATE NOCASE);

-- Optimize fetching assets needing thumbnails queue
CREATE INDEX IF NOT EXISTS idx_assets_thumbnails_queue ON assets(thumbnail_path, thumbnail_attempts) WHERE thumbnail_path IS NULL;
