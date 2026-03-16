-- Add added_at and modified_at columns to assets table for V1 parity
ALTER TABLE assets ADD COLUMN added_at TIMESTAMP;
ALTER TABLE assets ADD COLUMN modified_at TIMESTAMP;

-- Populate added_at with created_at as a fallback for existing rows
UPDATE assets SET added_at = created_at WHERE added_at IS NULL;
UPDATE assets SET modified_at = updated_at WHERE modified_at IS NULL;
