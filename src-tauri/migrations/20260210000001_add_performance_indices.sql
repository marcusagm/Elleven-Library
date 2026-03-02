-- Add performance indices for filtering and sorting

-- Index for media type and format filtering (e.g., 'Video', 'GIF', etc.)
CREATE INDEX IF NOT EXISTS idx_assets_format ON assets(format);
CREATE INDEX IF NOT EXISTS idx_assets_media_type ON assets(media_type);

-- Index for 'Date Added' sorting, common in the UI
CREATE INDEX IF NOT EXISTS idx_assets_added_at ON assets(added_at DESC);
