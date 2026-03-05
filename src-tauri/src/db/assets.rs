//! Asset management and metadata queries.

use super::Db;
use crate::db::models::AssetMetadata;

impl Db {
    /// Updates the star rating for a specific asset.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to update.
    /// * `rating` - The new rating to set for the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the update was successful.
    /// * `Err(sqlx::Error)` if the update failed.
    pub async fn update_asset_rating(&self, id: i64, rating: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE assets SET rating = ? WHERE id = ?", rating, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Updates the user notes for a specific asset.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to update.
    /// * `notes` - The new notes to set for the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the update was successful.
    /// * `Err(sqlx::Error)` if the update failed.
    pub async fn update_asset_notes(&self, id: i64, notes: String) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE assets SET notes = ? WHERE id = ?", notes, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Retrieves assets that do not have a thumbnail generated yet.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of assets to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(i64, String)>)` if the assets were retrieved successfully.
    /// * `Err(sqlx::Error)` if the assets could not be retrieved.
    pub async fn get_assets_needing_thumbnails(
        &self,
        limit: i32,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"SELECT id as "id!", path as "path!" FROM assets WHERE thumbnail_path IS NULL AND thumbnail_attempts < 3 LIMIT ?"#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.id as i64, r.path)).collect())
    }

    /// Retrieves specific assets needing thumbnails by their IDs.
    ///
    /// # Arguments
    ///
    /// * `ids` - A slice of asset IDs to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(i64, String)>)` if the assets were retrieved successfully.
    /// * `Err(sqlx::Error)` if the assets could not be retrieved.
    pub async fn get_assets_needing_thumbnails_by_ids(
        &self,
        ids: &[i64],
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, path FROM assets WHERE id IN ({}) AND thumbnail_path IS NULL AND thumbnail_attempts < 3",
            placeholders.join(",")
        );

        let mut query_builder = sqlx::query_as::<_, (i64, String)>(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    /// Increments the thumbnail attempts for a batch of assets before processing.
    ///
    /// # Arguments
    ///
    /// * `ids` - A slice of asset IDs to increment thumbnail attempts for.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the thumbnail attempts were incremented successfully.
    /// * `Err(sqlx::Error)` if the thumbnail attempts could not be incremented.
    pub async fn increment_thumbnail_attempts_batch(&self, ids: &[i64]) -> Result<(), sqlx::Error> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "UPDATE assets SET thumbnail_attempts = thumbnail_attempts + 1 WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut query_builder = sqlx::query(&query);
        for id in ids {
            query_builder = query_builder.bind(id);
        }

        query_builder.execute(&self.pool).await?;
        Ok(())
    }

    /// Records the last error message for a thumbnail.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to record the error for.
    /// * `error` - The error message to record.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the error was recorded successfully.
    /// * `Err(sqlx::Error)` if the error could not be recorded.
    pub async fn record_thumbnail_error(
        &self,
        asset_id: i64,
        error: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE assets SET thumbnail_last_error = ? WHERE id = ?",
            error,
            asset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Updates the path to the generated thumbnail for an asset and resets errors/attempts.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to update.
    /// * `path` - The path to the generated thumbnail.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the thumbnail path was updated successfully.
    /// * `Err(sqlx::Error)` if the thumbnail path could not be updated.
    pub async fn update_thumbnail_path(
        &self,
        asset_id: i64,
        path: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE assets SET thumbnail_path = ?, thumbnail_attempts = 0, thumbnail_last_error = NULL WHERE id = ?",
            path,
            asset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Clears the thumbnail path, effectively flagging it for regeneration, and resets errors/attempts.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to clear the thumbnail path for.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the thumbnail path was cleared successfully.
    /// * `Err(sqlx::Error)` if the thumbnail path could not be cleared.
    pub async fn clear_thumbnail_path(&self, asset_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE assets SET thumbnail_path = NULL, thumbnail_attempts = 0, thumbnail_last_error = NULL WHERE id = ?",
            asset_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Saves or updates a single asset record.
    ///
    /// # Arguments
    ///
    /// * `folder_id` - The ID of the folder to save the asset in.
    /// * `img` - The asset metadata to save.
    ///
    /// # Returns
    ///
    /// * `Ok((i64, Option<i64>, bool))` if the asset was saved successfully.
    /// * `Err(sqlx::Error)` if the asset could not be saved.
    pub async fn save_asset(
        &self,
        folder_id: i64,
        img: &crate::db::models::AssetMetadata,
    ) -> Result<(i64, Option<i64>, bool), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        self.save_asset_internal(&mut conn, folder_id, img).await
    }

    /// Batch saves multiple asset records within a transaction.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of tuples containing the folder ID and asset metadata to save.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the assets were saved successfully.
    /// * `Err(sqlx::Error)` if the assets could not be saved.
    pub async fn save_assets_batch(
        &self,
        items: Vec<(i64, crate::db::models::AssetMetadata)>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Prevent SQLITE_BUSY deadlocks by upgrading to a write lock IMMEDIATELY
        sqlx::query("INSERT INTO app_settings (key, value) VALUES ('_db_lock', '1') ON CONFLICT(key) DO UPDATE SET value = '1'")
            .execute(&mut *tx)
            .await
            .ok();

        for (folder_id, img) in items {
            if let Err(e) = self.save_asset_internal(&mut tx, folder_id, &img).await {
                tracing::error!("Failed to save asset in batch: {}", e);
            }
        }
        tx.commit().await?;
        Ok(())
    }

    /// Internal logic for saving/updating an asset, reusable for transactions.
    ///
    /// # Arguments
    ///
    /// * `conn` - The database connection to use.
    /// * `folder_id` - The ID of the folder to save the asset in.
    /// * `img` - The asset metadata to save.
    ///
    /// # Returns
    ///
    /// * `Ok((i64, Option<i64>, bool))` if the asset was saved successfully.
    /// * `Err(sqlx::Error)` if the asset could not be saved.
    async fn save_asset_internal(
        &self,
        conn: &mut sqlx::SqliteConnection,
        folder_id: i64,
        img: &crate::db::models::AssetMetadata,
    ) -> Result<(i64, Option<i64>, bool), sqlx::Error> {
        // 1. Check if path already exists
        let existing: Option<(i64, i64)> =
            sqlx::query_as("SELECT id, folder_id FROM assets WHERE path = ?")
                .bind(&img.path)
                .fetch_optional(&mut *conn)
                .await?;

        if let Some((id, old_fid)) = existing {
            sqlx::query!(
                "UPDATE assets SET
                    folder_id = ?, filename = ?, width = ?, height = ?, size = ?, format = ?, media_type = ?, modified_at = ?
                 WHERE path = ?",
                folder_id, img.filename, img.width, img.height, img.size, img.format, img.media_type, img.modified_at, img.path
            )
            .execute(&mut *conn)
            .await?;

            let old_fid_if_changed = if old_fid != folder_id {
                Some(old_fid)
            } else {
                None
            };
            return Ok((id, old_fid_if_changed, false));
        }

        // 2. Cross-root MOVE detection (fuzzy match by size and creation time if path is gone)
        let candidates: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT id, folder_id, path FROM assets WHERE size = ? AND created_at = ?",
        )
        .bind(img.size)
        .bind(img.created_at)
        .fetch_all(&mut *conn)
        .await?;

        for (id, old_fid, old_path) in candidates {
            if !std::path::Path::new(&old_path).exists() {
                sqlx::query!(
                    "UPDATE assets SET
                        path = ?, folder_id = ?, filename = ?, format = ?, media_type = ?, modified_at = ?
                     WHERE id = ?",
                    img.path,
                    folder_id,
                    img.filename,
                    img.format,
                    img.media_type,
                    img.modified_at,
                    id
                )
                .execute(&mut *conn)
                .await?;
                return Ok((id, Some(old_fid), false));
            }
        }

        // 3. True New File
        let res = sqlx::query!(
            "INSERT INTO assets (folder_id, path, filename, width, height, size, format, media_type, created_at, modified_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(path) DO UPDATE SET
                folder_id = excluded.folder_id,
                filename = excluded.filename,
                width = excluded.width,
                height = excluded.height,
                size = excluded.size,
                format = excluded.format,
                media_type = excluded.media_type,
                modified_at = excluded.modified_at",
            folder_id, img.path, img.filename, img.width, img.height, img.size, img.format, img.media_type, img.created_at, img.modified_at
        )
        .execute(conn)
        .await?;

        Ok((res.last_insert_rowid(), None, true))
    }

    /// Retrieve context (asset ID, folder ID, tags) for an asset.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<(i64, i64, Vec<i64>)>)` if the asset context was retrieved successfully.
    /// * `Err(sqlx::Error)` if the asset context could not be retrieved.
    pub async fn get_asset_context(
        &self,
        path: &str,
    ) -> Result<Option<(i64, i64, Vec<i64>)>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id as \"id!\", folder_id as \"folder_id!\" FROM assets WHERE path = ?",
            path
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let tags = sqlx::query!(
                "SELECT tag_id as \"tag_id!\" FROM asset_tags WHERE asset_id = ?",
                r.id
            )
            .fetch_all(&self.pool)
            .await?;

            let tag_ids = tags.into_iter().map(|t| t.tag_id).collect();
            Ok(Some((r.id, r.folder_id, tag_ids)))
        } else {
            Ok(None)
        }
    }

    /// Get size and creation date for comparison to detect file changes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<(i64, chrono::DateTime<chrono::Utc>)>)` if the file comparison data was retrieved successfully.
    /// * `Err(sqlx::Error)` if the file comparison data could not be retrieved.
    pub async fn get_file_comparison_data(
        &self,
        path: &str,
    ) -> Result<Option<(i64, chrono::DateTime<chrono::Utc>)>, sqlx::Error> {
        // Using explicit strings for cross-compatibility if needed, though Sqlite datetime usually maps well.
        let row: Option<(i64, String)> =
            sqlx::query_as("SELECT size, created_at FROM assets WHERE path = ?")
                .bind(path)
                .fetch_optional(&self.pool)
                .await?;

        if let Some((s, c_at)) = row {
            let created_dt = chrono::DateTime::parse_from_rfc3339(&c_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            Ok(Some((s, created_dt)))
        } else {
            Ok(None)
        }
    }

    /// Retrieves comparison data (size, modified_at) for all assets under a root path.
    /// Used for fast initial scanning.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root path to retrieve assets from.
    ///
    /// # Returns
    ///
    /// * `Ok(std::collections::HashMap<String, (i64, chrono::DateTime<chrono::Utc>)>)` if the file comparison data was retrieved successfully.
    /// * `Err(sqlx::Error)` if the file comparison data could not be retrieved.
    #[allow(clippy::type_complexity)]
    pub async fn get_all_files_comparison_data(
        &self,
        root_path: &str,
    ) -> Result<std::collections::HashMap<String, (i64, chrono::DateTime<chrono::Utc>)>, sqlx::Error>
    {
        let pattern = format!("{}%", root_path);
        let rows: Vec<(String, i64, String)> =
            sqlx::query_as("SELECT path, size, modified_at FROM assets WHERE path LIKE ?")
                .bind(pattern)
                .fetch_all(&self.pool)
                .await?;

        let mut map = std::collections::HashMap::with_capacity(rows.len());
        for (path, size, m_at) in rows {
            let dt = chrono::DateTime::parse_from_rfc3339(&m_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            map.insert(path, (size, dt));
        }
        Ok(map)
    }

    /// Deletes an asset record and returns its metadata context.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the asset to delete.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<(i64, i64, Vec<i64>)>)` if the asset context was retrieved successfully.
    /// * `Err(sqlx::Error)` if the asset context could not be retrieved.
    pub async fn delete_asset_by_path_returning_context(
        &self,
        path: &str,
    ) -> Result<Option<(i64, i64, Vec<i64>)>, sqlx::Error> {
        let context = self.get_asset_context(path).await?;

        if let Some((asset_id, _, _)) = context {
            sqlx::query!("DELETE FROM assets WHERE id = ?", asset_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(context)
    }

    /// Updates asset metadata due to a rename or move operation on the filesystem.
    ///
    /// # Arguments
    ///
    /// * `old_path` - The old path of the asset.
    /// * `new_path` - The new path of the asset.
    /// * `new_filename` - The new filename of the asset.
    /// * `new_folder_id` - The new folder ID of the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<(AssetMetadata, i64)>)` if the asset metadata was updated successfully.
    /// * `Err(sqlx::Error)` if the asset metadata could not be updated.
    #[allow(clippy::type_complexity)]
    pub async fn rename_asset(
        &self,
        old_path: &str,
        new_path: &str,
        new_filename: &str,
        new_folder_id: i64,
    ) -> Result<Option<(AssetMetadata, i64)>, sqlx::Error> {
        let row: Option<(i64, i64, i32, i32, i64, String, String, String, String, Option<String>, i32, Option<String>)> = sqlx::query_as(
            "SELECT id, folder_id, width, height, size, format, media_type, created_at, modified_at, thumbnail_path, rating, notes FROM assets WHERE path = ?"
        )
        .bind(old_path)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, old_folder_id, w, h, s, f, mt, c_at, _m_at, thumb, rating, notes)) = row {
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query!(
                "UPDATE assets SET path = ?, filename = ?, folder_id = ?, modified_at = ? WHERE id = ?",
                new_path, new_filename, new_folder_id, now, id
            )
            .execute(&self.pool)
            .await?;

            let created_dt = chrono::DateTime::parse_from_rfc3339(&c_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let modified_dt = chrono::Utc::now();

            Ok(Some((
                AssetMetadata {
                    id,
                    path: new_path.to_string(),
                    filename: new_filename.to_string(),
                    width: Some(w),
                    height: Some(h),
                    size: s,
                    created_at: created_dt,
                    modified_at: modified_dt,
                    thumbnail_path: thumb,
                    rating,
                    notes,
                    format: f,
                    media_type: mt,
                    added_at: None,
                    dominant_color: None,
                },
                old_folder_id,
            )))
        } else {
            Ok(None)
        }
    }
}
