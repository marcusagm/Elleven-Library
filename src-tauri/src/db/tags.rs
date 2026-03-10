//! Tag management and asset-tag relationship queries.

use super::Db;
use crate::db::models::{FolderCount, LibraryStats, Tag, TagCount};

impl Db {
    /// Creates a new tag in the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the tag.
    /// * `parent_id` - Optional ID of a parent tag for hierarchical organization.
    /// * `color` - Optional hex color code for the tag.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the database operation fails.
    pub async fn create_tag(
        &self,
        name: &str,
        parent_id: Option<i64>,
        color: Option<String>,
    ) -> Result<i64, sqlx::Error> {
        // Return existing tag ID instead of crashing via internal constraint
        if let Ok(Some(existing_tag)) = sqlx::query!("SELECT id FROM tags WHERE name = ?", name)
            .fetch_optional(&self.pool)
            .await
        {
            if let Some(id) = existing_tag.id {
                return Ok(id);
            }
        }

        let res = sqlx::query!(
            "INSERT INTO tags (name, parent_id, color) VALUES (?, ?, ?)",
            name,
            parent_id,
            color
        )
        .execute(&self.pool)
        .await?;

        Ok(res.last_insert_rowid())
    }

    /// Updates an existing tag's properties.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the tag doesn't exist or database fails.
    pub async fn update_tag(
        &self,
        id: i64,
        name: Option<String>,
        color: Option<String>,
        parent_id: Option<i64>,
        order_index: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let mut query = "UPDATE tags SET ".to_string();
        let mut updates = Vec::new();

        if name.is_some() {
            updates.push("name = ?");
        }
        if color.is_some() {
            updates.push("color = ?");
        }
        if parent_id.is_some() {
            updates.push("parent_id = ?");
        }
        if order_index.is_some() {
            updates.push("order_index = ?");
        }

        if updates.is_empty() {
            return Ok(());
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ?");

        let mut q = sqlx::query(&query);

        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(c) = color {
            q = q.bind(c);
        }
        if let Some(p) = parent_id {
            if p == 0 {
                q = q.bind(None::<i64>);
            } else {
                q = q.bind(p);
            }
        }
        if let Some(o) = order_index {
            q = q.bind(o);
        }

        q = q.bind(id);

        q.execute(&self.pool).await?;
        Ok(())
    }

    /// Deletes a tag and removes all its associations with assets.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the database fails.
    pub async fn delete_tag(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM tags WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Retrieves all tags from the database, ordered by their index and name.
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>, sqlx::Error> {
        let tags = sqlx::query_as!(
            Tag,
            "SELECT id as \"id!\", name, parent_id, color, order_index as \"order_index!\" FROM tags ORDER BY order_index ASC, name ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    /// Associates a tag with an asset.
    pub async fn add_tag_to_asset(&self, asset_id: i64, tag_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO asset_tags (asset_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
            asset_id,
            tag_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Removes an association between a tag and an asset.
    pub async fn remove_tag_from_asset(
        &self,
        asset_id: i64,
        tag_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM asset_tags WHERE asset_id = ? AND tag_id = ?",
            asset_id,
            tag_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Gets all tags associated with a specific asset.
    pub async fn get_tags_for_asset(&self, asset_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
        let tags = sqlx::query_as!(
            Tag,
            r#"SELECT t.id as "id!", t.name, t.parent_id, t.color, t.order_index as "order_index!"
               FROM tags t
               JOIN asset_tags it ON t.id = it.tag_id
               WHERE it.asset_id = ?
               ORDER BY t.order_index ASC, t.name ASC"#,
            asset_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    /// Batch associates multiple tags with multiple assets in a single transaction.
    pub async fn add_tags_to_assets_batch(
        &self,
        asset_ids: Vec<i64>,
        tag_ids: Vec<i64>,
    ) -> Result<(), sqlx::Error> {
        if asset_ids.is_empty() || tag_ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        // Prevent SQLITE_BUSY deadlocks by upgrading to a write lock IMMEDIATELY
        sqlx::query("INSERT INTO app_settings (key, value) VALUES ('_db_lock', '1') ON CONFLICT(key) DO UPDATE SET value = '1'")
            .execute(&mut *tx)
            .await
            .ok();

        for img_id in &asset_ids {
            for tag_id in &tag_ids {
                sqlx::query!(
                    "INSERT INTO asset_tags (asset_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
                    img_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Batch removes multiple tags from multiple assets in a single transaction.
    pub async fn remove_tags_from_assets_batch(
        &self,
        asset_ids: Vec<i64>,
        tag_ids: Vec<i64>,
    ) -> Result<(), sqlx::Error> {
        if asset_ids.is_empty() || tag_ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for img_id in &asset_ids {
            for tag_id in &tag_ids {
                sqlx::query!(
                    "DELETE FROM asset_tags WHERE asset_id = ? AND tag_id = ?",
                    img_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Batch replaces all tags for multiple assets in a single transaction.
    pub async fn replace_tags_for_assets_batch(
        &self,
        asset_ids: Vec<i64>,
        tag_ids: Vec<i64>,
    ) -> Result<(), sqlx::Error> {
        if asset_ids.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for img_id in &asset_ids {
            // Remove all existing tags
            sqlx::query!("DELETE FROM asset_tags WHERE asset_id = ?", img_id)
                .execute(&mut *tx)
                .await?;

            // Add new tags
            for tag_id in &tag_ids {
                sqlx::query!(
                    "INSERT INTO asset_tags (asset_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
                    img_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Calculates high-level library statistics.
    pub async fn get_library_stats(&self) -> Result<LibraryStats, sqlx::Error> {
        let total_assets = sqlx::query_scalar!("SELECT COUNT(*) FROM assets")
            .fetch_one(&self.pool)
            .await? as i64;

        let untagged_assets = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM assets WHERE id NOT IN (SELECT DISTINCT asset_id FROM asset_tags)"
        )
        .fetch_one(&self.pool)
        .await? as i64;

        let tag_counts = sqlx::query_as!(
            TagCount,
            "SELECT tag_id, COUNT(*) as count FROM asset_tags GROUP BY tag_id"
        )
        .fetch_all(&self.pool)
        .await?;

        let folder_counts = self
            .get_folder_counts_direct()
            .await?
            .into_iter()
            .map(|(folder_id, count)| FolderCount { folder_id, count })
            .collect();

        let folder_counts_recursive = self
            .get_folder_counts_recursive()
            .await?
            .into_iter()
            .map(|(folder_id, count)| FolderCount { folder_id, count })
            .collect();

        Ok(LibraryStats {
            total_assets,
            untagged_assets,
            tag_counts,
            folder_counts,
            folder_counts_recursive,
        })
    }
}
