-- FTS5 Trigram Search Index for high-quality fuzzy matching.
--
-- Replaces the legacy manual-trigram FTS approach from v1 with the native
-- SQLite trigram tokenizer (3.34+). This enables fast substring and
-- near-match queries over asset names and notes, ranked by BM25 relevance.
--
-- Architecture: content-synced FTS5 table backed by the `assets` table.
-- The FTS rowid maps directly to the implicit `assets.rowid`, avoiding
-- the TEXT primary key mismatch.

-- 1. Create the FTS5 virtual table with native trigram tokenizer.
CREATE VIRTUAL TABLE IF NOT EXISTS assets_fts USING fts5(
    name,
    notes,
    content='assets',
    content_rowid='rowid',
    tokenize='trigram'
);

-- 2. Populate index from existing data.
INSERT INTO assets_fts(assets_fts) VALUES('rebuild');

-- 3. Sync triggers to keep FTS consistent with the assets table.

-- On INSERT: index the new asset's searchable fields.
CREATE TRIGGER IF NOT EXISTS assets_fts_after_insert AFTER INSERT ON assets BEGIN
    INSERT INTO assets_fts(rowid, name, notes)
    VALUES (new.rowid, new.name, COALESCE(new.notes, ''));
END;

-- On DELETE: remove the asset's FTS entries using the special 'delete' command.
CREATE TRIGGER IF NOT EXISTS assets_fts_before_delete BEFORE DELETE ON assets BEGIN
    INSERT INTO assets_fts(assets_fts, rowid, name, notes)
    VALUES('delete', old.rowid, old.name, COALESCE(old.notes, ''));
END;

-- On UPDATE of searchable columns: re-index by deleting old and inserting new.
CREATE TRIGGER IF NOT EXISTS assets_fts_after_update AFTER UPDATE OF name, notes ON assets BEGIN
    INSERT INTO assets_fts(assets_fts, rowid, name, notes)
    VALUES('delete', old.rowid, old.name, COALESCE(old.notes, ''));
    INSERT INTO assets_fts(rowid, name, notes)
    VALUES (new.rowid, new.name, COALESCE(new.notes, ''));
END;
