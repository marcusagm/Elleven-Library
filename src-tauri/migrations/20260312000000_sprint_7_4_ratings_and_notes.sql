-- Sprint 7.4: Ratings and Notes
-- Adds support for asset ratings and personal notes.

-- 1. Update assets table
ALTER TABLE assets ADD COLUMN rating INTEGER DEFAULT 0;
ALTER TABLE assets ADD COLUMN notes TEXT DEFAULT '';

-- 2. Update asset_metadata_envelope table for consistency
ALTER TABLE asset_metadata_envelope ADD COLUMN rating INTEGER DEFAULT 0;
ALTER TABLE asset_metadata_envelope ADD COLUMN notes TEXT DEFAULT '';
