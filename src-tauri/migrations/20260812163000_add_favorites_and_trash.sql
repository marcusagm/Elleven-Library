-- Add favorites and trash columns
ALTER TABLE assets ADD COLUMN is_favorite BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE assets ADD COLUMN deleted_at DATETIME;
