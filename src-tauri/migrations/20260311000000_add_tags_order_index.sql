-- Sprint 7.1: Add order_index to tags table
-- Adds the order_index column that existed in V1 for tag ordering in the UI.

ALTER TABLE tags ADD COLUMN order_index INTEGER DEFAULT 0;
