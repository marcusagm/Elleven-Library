//! Color palette persistence and search queries.
//!
//! This module handles storing extracted color palettes from image assets,
//! querying colors for display in the inspector, and searching assets
//! by color proximity using CIE-76 distance in the LAB color space.

use super::models::AssetColor;
use super::Db;

impl Db {
    /// Inserts a batch of extracted colors for a given asset within a transaction.
    ///
    /// Replaces any existing colors for the asset to support re-extraction.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The database ID of the asset.
    /// * `colors` - Slice of `AssetColor` structs to persist.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the transaction or any insert fails.
    pub async fn insert_asset_colors(
        &self,
        asset_id: i64,
        colors: &[AssetColor],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query("DELETE FROM asset_colors WHERE asset_id = ?")
            .bind(asset_id)
            .execute(&mut *transaction)
            .await?;

        for color in colors {
            sqlx::query(
                "INSERT INTO asset_colors (asset_id, hex_color, lab_lightness, lab_green_red, lab_blue_yellow, percentage, rank)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(asset_id)
            .bind(&color.hex_color)
            .bind(color.lab_lightness)
            .bind(color.lab_green_red)
            .bind(color.lab_blue_yellow)
            .bind(color.percentage)
            .bind(color.rank)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    /// Retrieves all extracted colors for a specific asset, ordered by rank.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query execution fails.
    pub async fn get_asset_colors(&self, asset_id: i64) -> Result<Vec<AssetColor>, sqlx::Error> {
        let rows: Vec<AssetColor> = sqlx::query_as(
            "SELECT id, asset_id, hex_color, lab_lightness, lab_green_red, lab_blue_yellow, percentage, rank
             FROM asset_colors
             WHERE asset_id = ?
             ORDER BY rank ASC",
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Deletes all extracted colors for a specific asset.
    ///
    /// Used before re-extraction or when an asset is being reprocessed.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the deletion fails.
    pub async fn delete_asset_colors(&self, asset_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM asset_colors WHERE asset_id = ?")
            .bind(asset_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Updates the dominant (most prominent) color for an asset.
    ///
    /// This is a denormalized field on the `assets` table for fast read access
    /// without JOINing the `asset_colors` table.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_dominant_color(
        &self,
        asset_id: i64,
        hex_color: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE assets SET dominant_color = ? WHERE id = ?")
            .bind(hex_color)
            .bind(asset_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Searches for asset IDs that have at least one color within the given
    /// CIE-76 distance threshold from the target LAB color.
    ///
    /// CIE-76 uses Euclidean distance in the CIELAB color space:
    /// `ΔE = sqrt((L₁-L₂)² + (a₁-a₂)² + (b₁-b₂)²)`
    ///
    /// # Arguments
    ///
    /// * `target_lightness` - Target L* component (0–100).
    /// * `target_green_red` - Target a* component (-128 to 127).
    /// * `target_blue_yellow` - Target b* component (-128 to 127).
    /// * `threshold` - Maximum ΔE (distance) to consider a match.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn search_assets_by_color(
        &self,
        target_lightness: f64,
        target_green_red: f64,
        target_blue_yellow: f64,
        threshold: f64,
    ) -> Result<Vec<i64>, sqlx::Error> {
        let threshold_squared = threshold * threshold;

        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT DISTINCT asset_id FROM asset_colors
             WHERE (
                 (lab_lightness - ?) * (lab_lightness - ?) +
                 (lab_green_red - ?) * (lab_green_red - ?) +
                 (lab_blue_yellow - ?) * (lab_blue_yellow - ?)
             ) < ?",
        )
        .bind(target_lightness)
        .bind(target_lightness)
        .bind(target_green_red)
        .bind(target_green_red)
        .bind(target_blue_yellow)
        .bind(target_blue_yellow)
        .bind(threshold_squared)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Retrieves asset IDs that have had color extraction performed.
    ///
    /// Returns IDs of `MediaType::Image` assets that have a `thumbnail_path`
    /// but no entry in `asset_colors`. Used by the re-extraction command.
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn get_assets_needing_color_extraction(
        &self,
        limit: i32,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let rows: Vec<(i64, String)> = sqlx::query_as(
            "SELECT a.id, a.thumbnail_path FROM assets a
             LEFT JOIN asset_colors ac ON a.id = ac.asset_id
             WHERE a.media_type = 'Image'
             AND a.thumbnail_path IS NOT NULL
             AND ac.id IS NULL
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
