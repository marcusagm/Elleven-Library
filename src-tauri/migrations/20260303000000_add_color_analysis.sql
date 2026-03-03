-- Color Analysis: stores extracted color palette for image assets.
-- Each image asset can have up to 16 colors extracted via k-means clustering
-- in the CIE-LAB color space for perceptually accurate color search.

ALTER TABLE assets ADD COLUMN dominant_color TEXT;

CREATE TABLE IF NOT EXISTS asset_colors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    hex_color TEXT NOT NULL,
    lab_lightness REAL NOT NULL,
    lab_green_red REAL NOT NULL,
    lab_blue_yellow REAL NOT NULL,
    percentage REAL NOT NULL,
    rank INTEGER NOT NULL,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_asset_colors_asset ON asset_colors(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_colors_lab ON asset_colors(lab_lightness, lab_green_red, lab_blue_yellow);
