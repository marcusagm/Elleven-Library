-- Sprint 4.1: Thumbnail Worker Pool & Priority Queue
-- Adding thumbnail_path column to v2_assets

ALTER TABLE v2_assets ADD COLUMN thumbnail_path TEXT;

-- Index for background querying (FIFO)
CREATE INDEX IF NOT EXISTS idx_v2_assets_thumbnail_path ON v2_assets(thumbnail_path);
