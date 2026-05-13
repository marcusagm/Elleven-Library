use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetColor, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag};
use crate::core::repository::AssetQueryHandler;
use crate::infra::database::models::AssetSummaryDb;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::sync::Arc;

/// SQLite implementation of the AssetQueryHandler port.
pub struct SqliteAssetQueries {
    /// The database connection pool.
    pool: SqlitePool,
    /// The central "Cartório" for format definitions.
    registry: Arc<crate::core::formats::registry::FormatRegistry>,
}

// Helper methods
impl SqliteAssetQueries {
    /// Creates a new instance with the given connection pool.
    pub fn new(
        pool: SqlitePool,
        registry: Arc<crate::core::formats::registry::FormatRegistry>,
    ) -> Self {
        Self { pool, registry }
    }
}

/// Implementation of the AssetQueryHandler port for SqliteAssetQueries.
#[async_trait]
impl AssetQueryHandler for SqliteAssetQueries {

    /// Retrieves a list of asset IDs that are missing thumbnails.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of asset IDs to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` if the asset IDs were found successfully.
    /// * `Err(sqlx::Error)` if the asset IDs could not be found.
    async fn get_assets_needing_thumbnails(&self, limit: u32) -> AppResult<Vec<String>> {
        let limit_i64 = limit as i64;
        let rows = sqlx::query!(
            r#"SELECT id as "id!" FROM assets WHERE thumbnail_path IS NULL AND state != 'Thumbnailed' LIMIT ?"#,
            limit_i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect::<Vec<String>>())
    }
    /// Finds all assets in the database.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Asset>)` if the assets were found successfully.
    /// * `Err(sqlx::Error)` if the assets could not be found.
    async fn find_all(&self) -> AppResult<Vec<Asset>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                m.width as "width: i64",
                m.height as "height: i64",
                a.rating as "rating: i64",
                a.notes as "notes?",
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| crate::infra::database::models::AssetDb {
                id: r.id,
                name: r.name,
                path: r.path,
                state: r.state,
                format_type: r.format_type,
                family: r.family,
                file_size: r.file_size,
                created_at: r.created_at,
                modified_at: r.modified_at,
                added_at: r.added_at,
                updated_at: r.updated_at,
                folder_id: r.folder_id,
                thumbnail_path: r.thumbnail_path,
                rating: r.rating,
                notes: r.notes,
                width: r.width,
                height: r.height,
                duration_secs: r.duration_secs,
                technical_payload: r.technical_payload,
                semantic_payload: r.semantic_payload,
                dominant_color: r.dominant_color,
            })
            .map(Into::into)
            .collect())
    }

    /// Gets an asset by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Asset>)` if the asset was found successfully.
    /// * `Err(sqlx::Error)` if the asset could not be found.
    async fn get_by_id(&self, id: &str) -> AppResult<Option<Asset>> {
        let row = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                a.rating as "rating: i64",
                a.notes as "notes?",
                m.width as "width: i64",
                m.height as "height: i64",
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| crate::infra::database::models::AssetDb {
            id: r.id,
            name: r.name,
            path: r.path,
            state: r.state,
            format_type: r.format_type,
            family: r.family,
            file_size: r.file_size,
            created_at: r.created_at,
            modified_at: r.modified_at,
            added_at: r.added_at,
            updated_at: r.updated_at,
            folder_id: r.folder_id,
            thumbnail_path: r.thumbnail_path,
            rating: r.rating,
            notes: r.notes,
            width: r.width,
            height: r.height,
            duration_secs: r.duration_secs,
            technical_payload: r.technical_payload,
            semantic_payload: r.semantic_payload,
            dominant_color: r.dominant_color,
        }).map(Into::into))
    }

    /// Lists assets with pagination and filtering.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter to apply to the assets.
    /// * `page` - The pagination parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
    /// * `Err(sqlx::Error)` if the assets could not be found.
    async fn list_paginated(
        &self,
        filter: AssetFilter,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT 
                a.id as id, a.name as name, a.path as path, a.state as state, 
                a.format_type as format_type, a.family as family, 
                a.created_at as created_at, 
                a.modified_at as modified_at,
                a.added_at as added_at,
                a.updated_at as updated_at, 
                a.folder_id as folder_id, a.thumbnail_path as thumbnail_path, 
                a.file_size as file_size, 
                m.width as width, m.height as height, 
                a.rating as rating, a.notes as notes 
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE 1=1 
            "#,
        );

        if let Some(family) = filter.family {
            query_builder.push(" AND a.family = ");
            query_builder.push_bind(family);
        }

        if let Some(state) = filter.state {
            query_builder.push(" AND a.state = ");
            query_builder.push_bind(state.to_string());
        }

        if let Some(search) = filter.search_query {
            query_builder.push(" AND (a.name LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(" OR a.path LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(")");
        }

        if let Some(folder_id) = filter.folder_id {
            if filter.recursive.unwrap_or(false) {
                query_builder.push(" AND a.folder_id IN (WITH RECURSIVE subfolders AS (SELECT id FROM folders WHERE id = ");
                query_builder.push_bind(folder_id);
                query_builder.push(" UNION ALL SELECT f.id FROM folders f JOIN subfolders ON f.parent_id = subfolders.id) SELECT id FROM subfolders)");
            } else {
                query_builder.push(" AND a.folder_id = ");
                query_builder.push_bind(folder_id);
            }
        }

        if let Some(tags) = filter.tags {
            if !tags.is_empty() {
                query_builder.push(" AND a.id IN (SELECT asset_id FROM asset_tags WHERE tag_id IN (");
                let mut first = true;
                for tag in tags {
                    if !first {
                        query_builder.push(", ");
                    }
                    query_builder.push_bind(tag);
                    first = false;
                }
                query_builder.push("))");
            }
        }

        if let Some(untagged) = filter.untagged {
            if untagged {
                query_builder.push(" AND a.id NOT IN (SELECT asset_id FROM asset_tags)");
            }
        }

        // Ordering as per Sprint decision: created_at DESC, name ASC
        query_builder.push(" ORDER BY a.created_at DESC, a.name ASC ");

        // Pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page.limit() as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(page.offset() as i64);

        let rows = query_builder
            .build_query_as::<AssetSummaryDb>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(AssetSummaryDto::from).collect())
    }

    /// Lists folders in the database.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The ID of the parent folder.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Folder>)` if the folders were found successfully.
    /// * `Err(sqlx::Error)` if the folders could not be found.
    async fn list_folders(
        &self,
        parent_id: Option<String>,
    ) -> AppResult<Vec<crate::core::models::Folder>> {
        let rows = if let Some(parent) = parent_id {
            sqlx::query_as!(
                crate::infra::database::models::FolderDb,
                r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM folders WHERE parent_id = ?"#,
                parent
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                crate::infra::database::models::FolderDb,
                r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM folders WHERE parent_id IS NULL"#
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(Folder::from).collect())
    }

    /// Gets a folder by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the folder to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Folder>)` if the folder was found successfully.
    /// * `Err(sqlx::Error)` if the folder could not be found.
    async fn get_folder_by_id(&self, id: &str) -> AppResult<Option<crate::core::models::Folder>> {
        let row = sqlx::query_as!(
            crate::infra::database::models::FolderDb,
            r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM folders WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Folder::from))
    }

    /// Lists all folders (entire hierarchy).
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Folder>)` if the folders were found successfully.
    /// * `Err(sqlx::Error)` if the folders could not be found.
    async fn list_all_subfolders(&self) -> AppResult<Vec<Folder>> {
        let rows = sqlx::query_as!(
            crate::infra::database::models::FolderDb,
            r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM folders ORDER BY path"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Folder::from).collect())
    }

    /// Returns the asset counts for all folders (recursive).
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, i64)>)` if the counts were found successfully.
    /// * `Err(sqlx::Error)` if the counts could not be found.
    async fn get_subfolder_asset_counts(&self) -> AppResult<Vec<(String, i64)>> {
        let rows = sqlx::query!(
            r#"
            WITH RECURSIVE 
              hierarchy_tree(root_id, child_id) AS (
                SELECT id, id FROM folders
                UNION ALL
                SELECT ht.root_id, f.id
                FROM folders f
                JOIN hierarchy_tree ht ON f.parent_id = ht.child_id
              ),
              leaf_counts AS (
                SELECT folder_id, COUNT(*) as cnt FROM assets GROUP BY folder_id
              )
            SELECT ht.root_id as "folder_id!", SUM(COALESCE(lc.cnt, 0)) as "count!"
            FROM hierarchy_tree ht
            LEFT JOIN leaf_counts lc ON lc.folder_id = ht.child_id
            GROUP BY ht.root_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.folder_id, r.count)).collect())
    }

    /// Returns the asset counts for root locations.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, i64)>)` if the counts were found successfully.
    /// * `Err(sqlx::Error)` if the counts could not be found.
    async fn get_location_root_counts(&self) -> AppResult<Vec<(String, i64)>> {
        let rows = sqlx::query!(
            r#"
            WITH RECURSIVE 
              hierarchy_tree(root_id, child_id) AS (
                SELECT id, id FROM folders WHERE parent_id IS NULL
                UNION ALL
                SELECT ht.root_id, f.id
                FROM folders f
                JOIN hierarchy_tree ht ON f.parent_id = ht.child_id
              ),
              leaf_counts AS (
                SELECT folder_id, COUNT(*) as cnt FROM assets GROUP BY folder_id
              )
            SELECT ht.root_id as "folder_id!", SUM(COALESCE(lc.cnt, 0)) as "count!"
            FROM hierarchy_tree ht
            LEFT JOIN leaf_counts lc ON lc.folder_id = ht.child_id
            GROUP BY ht.root_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.folder_id, r.count)).collect())
    }

    /// Returns thumbnail paths for all assets in a folder and its subfolders.
    ///
    /// # Arguments
    ///
    /// * `folder_id` - The ID of the folder to retrieve thumbnails for.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` if the thumbnails were found successfully.
    /// * `Err(sqlx::Error)` if the thumbnails could not be found.
    async fn get_folder_thumbnails(&self, folder_id: &str) -> AppResult<Vec<String>> {
        let rows = sqlx::query!(
            r#"
            WITH RECURSIVE family AS (
                SELECT id FROM folders WHERE id = ?
                UNION ALL
                SELECT f.id FROM folders f JOIN family ON f.parent_id = family.id
            )
            SELECT thumbnail_path as "thumbnail_path!" FROM assets
            WHERE folder_id IN family AND thumbnail_path IS NOT NULL
            "#,
            folder_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.thumbnail_path).collect())
    }

    /// Lists tags in the database.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Tag>)` if the tags were found successfully.
    /// * `Err(sqlx::Error)` if the tags could not be found.
    async fn list_tags(&self) -> AppResult<Vec<crate::core::models::Tag>> {
        let rows = sqlx::query_as!(
            crate::infra::database::models::TagDb,
            r#"SELECT id as "id!", name as "name!", color, parent_id, order_index as "order_index!" FROM tags ORDER BY order_index ASC, name ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Tag::from).collect())
    }

    /// Searches for assets based on the provided criteria.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The search criteria.
    /// * `page` - The pagination parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
    /// * `Err(sqlx::Error)` if the assets could not be found.
    async fn search_assets(
        &self,
        criteria: crate::core::models::SearchCriteria,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>> {
        use crate::infra::database::search_builder::build_search_where_clause;

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT DISTINCT
                a.id as id, a.name as name, a.path as path, a.state as state, 
                a.format_type as format_type, a.family as family, 
                a.created_at as created_at, 
                a.modified_at as modified_at,
                a.added_at as added_at,
                a.updated_at as updated_at, 
                a.folder_id as folder_id, a.thumbnail_path as thumbnail_path, 
                a.file_size as file_size, 
                m.width as width, m.height as height, 
                a.rating as rating, a.notes as notes 
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE 1=1 AND 
            "#,
        );

        build_search_where_clause(&criteria.root_group, &mut query_builder, &self.registry);

        // Ordering as per project standard
        query_builder.push(" ORDER BY a.created_at DESC, a.name ASC ");

        // Pagination
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page.limit() as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(page.offset() as i64);

        let rows = query_builder
            .build_query_as::<AssetSummaryDb>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(AssetSummaryDto::from).collect())
    }

    async fn get_search_count(
        &self,
        criteria: crate::core::models::SearchCriteria,
    ) -> AppResult<i64> {
        use crate::infra::database::search_builder::build_search_where_clause;
 
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT COUNT(DISTINCT a.id)
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE 1=1 AND
            "#
        );
        build_search_where_clause(&criteria.root_group, &mut query_builder, &self.registry);

        let row = query_builder.build().fetch_one(&self.pool).await?;
        let count: i64 = sqlx::Row::get(&row, 0);

        Ok(count)
    }


    /// Retrieves a single asset by its unique ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Asset)` if the asset was found successfully.
    /// * `Err(sqlx::Error)` if the asset could not be found.
    async fn get_asset_by_id(&self, id: &str) -> AppResult<Asset> {
        let row = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                a.rating as "rating: i64",
                a.notes as "notes?",
                m.width as "width: i64",
                m.height as "height: i64",
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::core::error::AppError::NotFound(id.to_string()))?;

        let asset_db = crate::infra::database::models::AssetDb {
            id: row.id,
            name: row.name,
            path: row.path,
            state: row.state,
            format_type: row.format_type,
            family: row.family,
            file_size: row.file_size,
            created_at: row.created_at,
            modified_at: row.modified_at,
            added_at: row.added_at,
            updated_at: row.updated_at,
            folder_id: row.folder_id,
            thumbnail_path: row.thumbnail_path,
            rating: row.rating,
            notes: row.notes,
            width: row.width,
            height: row.height,
            duration_secs: row.duration_secs,
            technical_payload: row.technical_payload,
            semantic_payload: row.semantic_payload,
            dominant_color: row.dominant_color,
        };

        Ok(asset_db.into())
    }

    /// Retrieves a map of path -> (file_size, updated_at) for all assets under a root path.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root path to search for assets.
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, (i64, DateTime<Utc>)>)` if the assets were found successfully.
    /// * `Err(sqlx::Error)` if the assets could not be found.
    async fn get_all_files_comparison_data(
        &self,
        root_path: &str,
    ) -> AppResult<std::collections::HashMap<String, (i64, DateTime<Utc>)>> {
        let pattern = format!("{}%", root_path);
        let rows = sqlx::query!(
            r#"SELECT path as "path!", file_size as "file_size!", modified_at as "modified_at!: DateTime<Utc>" FROM assets WHERE path LIKE ?"#,
            pattern
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.path, (r.file_size, r.modified_at)))
            .collect())
    }

    /// Retrieves all tags associated with a specific asset.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The unique identifier of the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Tag>)` if the tags were found successfully.
    /// * `Err(sqlx::Error)` if the query fails.
    async fn get_tags_for_asset(&self, asset_id: &str) -> AppResult<Vec<crate::core::models::Tag>> {
        let rows = sqlx::query_as!(
            crate::infra::database::models::TagDb,
            r#"SELECT t.id as "id!", t.name as "name!", t.color, t.parent_id, t.order_index as "order_index!"
               FROM tags t
               JOIN asset_tags at ON t.id = at.tag_id
               WHERE at.asset_id = ?
               ORDER BY t.order_index ASC, t.name ASC"#,
            asset_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Tag::from).collect())
    }

    /// Lists all smart folders.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SmartFolder>)` if successful.
    /// * `Err(sqlx::Error)` on query failure.
    async fn list_smart_folders(&self) -> AppResult<Vec<crate::core::models::SmartFolder>> {
        let rows = sqlx::query!(
            r#"SELECT id as "id!", name as "name!", query_json as "query_json!", created_at as "created_at?: chrono::DateTime<chrono::Utc>", updated_at as "updated_at?: chrono::DateTime<chrono::Utc>" FROM smart_folders ORDER BY name ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut folders = Vec::new();
        for r in rows {
            folders.push(crate::core::models::SmartFolder {
                id: r.id,
                name: r.name,
                query_json: r.query_json,
                created_at: r.created_at.unwrap_or_else(chrono::Utc::now),
                updated_at: r.updated_at.unwrap_or_else(chrono::Utc::now),
            });
        }
        
        Ok(folders)
    }

    /// Gets the total count of assets matching the specified filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - The given filters.
    ///
    /// # Returns
    ///
    /// * `Ok(i64)` asset count.
    /// * `Err(sqlx::Error)` on query failure.
    async fn get_asset_count(&self, filter: AssetFilter) -> AppResult<i64> {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) as count FROM assets WHERE 1=1 ");

        if let Some(family) = filter.family {
            query_builder.push(" AND family = ");
            query_builder.push_bind(family);
        }

        if let Some(state) = filter.state {
            query_builder.push(" AND state = ");
            query_builder.push_bind(state.to_string());
        }

        if let Some(search) = filter.search_query {
            query_builder.push(" AND (name LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(" OR path LIKE ");
            query_builder.push_bind(format!("%{}%", search));
            query_builder.push(")");
        }

        if let Some(folder_id) = filter.folder_id {
            if filter.recursive.unwrap_or(false) {
                query_builder.push(" AND folder_id IN (WITH RECURSIVE subfolders AS (SELECT id FROM folders WHERE id = ");
                query_builder.push_bind(folder_id);
                query_builder.push(" UNION ALL SELECT f.id FROM folders f JOIN subfolders ON f.parent_id = subfolders.id) SELECT id FROM subfolders)");
            } else {
                query_builder.push(" AND folder_id = ");
                query_builder.push_bind(folder_id);
            }
        }

        if let Some(tags) = filter.tags {
            if !tags.is_empty() {
                query_builder.push(" AND id IN (SELECT asset_id FROM asset_tags WHERE tag_id IN (");
                let mut first = true;
                for tag in tags {
                    if !first {
                        query_builder.push(", ");
                    }
                    query_builder.push_bind(tag);
                    first = false;
                }
                query_builder.push("))");
            }
        }

        if let Some(untagged) = filter.untagged {
            if untagged {
                query_builder.push(" AND id NOT IN (SELECT asset_id FROM asset_tags)");
            }
        }

        let row = query_builder.build().fetch_one(&self.pool).await?;
        let count: i64 = sqlx::Row::get(&row, 0);

        Ok(count)
    }

    /// Gets library statistics (total assets, folders, tags, size).
    ///
    /// # Returns
    ///
    /// * `Ok(LibraryStats)` with aggregated data.
    /// * `Err(sqlx::Error)` on query failure.
    async fn get_library_stats(&self) -> AppResult<crate::core::models::LibraryStats> {
        let stats_row = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM assets) as "total_assets!: i64",
                (SELECT COUNT(*) FROM folders) as "total_folders!: i64",
                (SELECT COUNT(*) FROM tags) as "total_tags!: i64",
                (SELECT COALESCE(SUM(file_size), 0) FROM assets) as "total_size_bytes!: i64",
                (SELECT COUNT(*) FROM assets WHERE id NOT IN (SELECT asset_id FROM asset_tags)) as "untagged_assets!: i64"
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let tag_counts_rows = sqlx::query!(
            r#"
            SELECT tag_id, COUNT(asset_id) as "count!: i64"
            FROM asset_tags
            GROUP BY tag_id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let tag_counts = tag_counts_rows
            .into_iter()
            .map(|r| crate::core::models::TagCount {
                tag_id: r.tag_id,
                count: r.count,
            })
            .collect();

        let folder_counts_rows = sqlx::query!(
            r#"
            SELECT folder_id as "folder_id: String", COUNT(id) as "count!: i64"
            FROM assets
            WHERE folder_id IS NOT NULL
            GROUP BY folder_id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let folder_counts: Vec<crate::core::models::FolderCount> = folder_counts_rows
            .into_iter()
            .filter_map(|r| r.folder_id.map(|id| crate::core::models::FolderCount {
                folder_id: id,
                count: r.count,
            }))
            .collect();

        // Calculate recursive counts in-memory instead of slow recursive CTE
        let all_folders = sqlx::query!(
            r#"SELECT id as "id!", parent_id as "parent_id?" FROM folders"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut parent_map: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
        for row in all_folders {
            parent_map.insert(row.id, row.parent_id);
        }

        let mut recursive_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for fc in &folder_counts {
            // Add to itself
            *recursive_counts.entry(fc.folder_id.clone()).or_insert(0) += fc.count;
            
            // Add to all parents
            let mut current_parent = parent_map.get(&fc.folder_id).cloned().flatten();
            while let Some(parent_id) = current_parent {
                *recursive_counts.entry(parent_id.clone()).or_insert(0) += fc.count;
                current_parent = parent_map.get(&parent_id).cloned().flatten();
            }
        }

        let folder_counts_recursive = recursive_counts
            .into_iter()
            .map(|(folder_id, count)| crate::core::models::FolderCount { folder_id, count })
            .collect();

        Ok(crate::core::models::LibraryStats {
            total_assets: stats_row.total_assets,
            total_folders: stats_row.total_folders,
            total_tags: stats_row.total_tags,
            total_size_bytes: stats_row.total_size_bytes,
            untagged_assets: stats_row.untagged_assets,
            tag_counts,
            folder_counts,
            folder_counts_recursive: Some(folder_counts_recursive),
        })
    }

    /// Retrieves all colors extracted for a specific asset.
    async fn get_asset_colors(&self, asset_id: &str) -> AppResult<Vec<crate::core::models::AssetColor>> {
        let rows = sqlx::query!(
            r#"
            SELECT id as "id!", hex_color, lab_lightness, lab_green_red, lab_blue_yellow, percentage, rank
            FROM asset_colors
            WHERE asset_id = ?
            ORDER BY rank ASC
            "#,
            asset_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AssetColor {
                id: Some(r.id),
                hex_color: r.hex_color,
                lab_lightness: r.lab_lightness,
                lab_green_red: r.lab_green_red,
                lab_blue_yellow: r.lab_blue_yellow,
                percentage: r.percentage,
                rank: r.rank as i32,
            })
            .collect())
    }

    async fn find_folder_by_path(&self, path: &str) -> AppResult<Option<String>> {
        let row = sqlx::query!(
            r#"SELECT id as "id!" FROM folders WHERE path = ? COLLATE NOCASE"#,
            path
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.id))
    }

    async fn find_asset_by_path(&self, path: &str) -> AppResult<Option<Asset>> {
        let row = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                a.rating as "rating: i64",
                a.notes as "notes?",
                m.width as "width: i64",
                m.height as "height: i64",
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.path = ? COLLATE NOCASE
            "#,
            path
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| crate::infra::database::models::AssetDb {
            id: r.id,
            name: r.name,
            path: r.path,
            state: r.state,
            format_type: r.format_type,
            family: r.family,
            file_size: r.file_size,
            created_at: r.created_at,
            modified_at: r.modified_at,
            added_at: r.added_at,
            updated_at: r.updated_at,
            folder_id: r.folder_id,
            thumbnail_path: r.thumbnail_path,
            rating: r.rating,
            notes: r.notes,
            width: r.width,
            height: r.height,
            duration_secs: r.duration_secs,
            technical_payload: r.technical_payload,
            semantic_payload: r.semantic_payload,
            dominant_color: r.dominant_color,
        }).map(Into::into))
    }

    async fn find_assets_by_size(&self, size_bytes: u64, state: Option<crate::core::models::AssetState>) -> AppResult<Vec<Asset>> {
        let size_i64 = size_bytes as i64;
        let state_str = state.map(|s| s.to_string());

        let rows = sqlx::query!(
            r#"
            SELECT 
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!", 
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!", 
                a.created_at as "created_at: DateTime<Utc>", 
                a.modified_at as "modified_at: DateTime<Utc>", 
                a.added_at as "added_at: DateTime<Utc>", 
                a.updated_at as "updated_at: DateTime<Utc>", 
                a.folder_id as "folder_id?", a.thumbnail_path as "thumbnail_path?", 
                a.rating as "rating?", a.notes as "notes?", 
                m.width as "width?", m.height as "height?", 
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.file_size = ? 
              AND (? IS NULL OR a.state = ?)
            "#,
            size_i64,
            state_str,
            state_str
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| crate::infra::database::models::AssetDb {
            id: r.id,
            name: r.name,
            path: r.path,
            state: r.state,
            format_type: r.format_type,
            family: r.family,
            file_size: r.file_size,
            created_at: r.created_at,
            modified_at: r.modified_at,
            added_at: r.added_at,
            updated_at: r.updated_at,
            folder_id: r.folder_id,
            thumbnail_path: r.thumbnail_path,
            rating: r.rating,
            notes: r.notes,
            width: r.width,
            height: r.height,
            duration_secs: r.duration_secs,
            technical_payload: r.technical_payload,
            semantic_payload: r.semantic_payload,
            dominant_color: r.dominant_color,
        }).map(Into::into).collect())
    }

    async fn get_assets_needing_repair(&self) -> AppResult<Vec<Asset>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                a.rating as "rating: i64",
                a.notes as "notes?",
                m.width as "width: i64",
                m.height as "height: i64",
                m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.format_type = 'unknown' OR a.thumbnail_path IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| crate::infra::database::models::AssetDb {
                id: r.id,
                name: r.name,
                path: r.path,
                state: r.state,
                format_type: r.format_type,
                family: r.family,
                file_size: r.file_size,
                created_at: r.created_at,
                modified_at: r.modified_at,
                added_at: r.added_at,
                updated_at: r.updated_at,
                folder_id: r.folder_id,
                thumbnail_path: r.thumbnail_path,
                rating: r.rating,
                notes: r.notes,
                width: r.width,
                height: r.height,
                duration_secs: r.duration_secs,
                technical_payload: r.technical_payload,
                semantic_payload: r.semantic_payload,
                dominant_color: r.dominant_color,
            })
            .map(Into::into)
            .collect())
    }

    async fn adopt_orphaned_children(&self, parent_id: &str, parent_path: &str) -> AppResult<()> {
        let pattern = format!("{}/%", parent_path.trim_end_matches('/'));
        
        // Update all folders that:
        // 1. Are currently roots (parent_id IS NULL)
        // 2. Are not the parent itself
        // 3. Are physically inside the parent folder path
        sqlx::query!(
            r#"
            UPDATE folders 
            SET parent_id = ? 
            WHERE parent_id IS NULL 
              AND id != ?
              AND path LIKE ?
            "#,
            parent_id,
            parent_id,
            pattern
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Tests for the SqliteAssetQueries struct.
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Tests the asset insertion and query functionality.
    #[tokio::test]
    async fn test_asset_pagination_and_filters() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory DB");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Insert multiple mock assets
        for i in 1..=10 {
            sqlx::query("INSERT INTO assets (id, name, path, state, format_type, family, file_size, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(format!("id-{}", i))
                .bind(format!("file-{}.png", i))
                .bind(format!("/tmp/file-{}.png", i))
                .bind("Idle")
                .bind("image/png")
                .bind(if i % 2 == 0 { "IMAGE" } else { "VIDEO" })
                .bind(1024)
                .bind(Utc::now())
                .execute(&pool)
                .await
                .expect("Failed to insert mock asset");
        }

        let registry = std::sync::Arc::new(crate::core::formats::build_format_registry());
        let handler = SqliteAssetQueries::new(pool, registry);

        // Test basic pagination
        let page = PageParams {
            page: 1,
            page_size: 5,
        };
        let assets: Vec<AssetSummaryDto> = handler
            .list_paginated(AssetFilter::default(), page)
            .await
            .expect("Failed to list assets");
        assert_eq!(assets.len(), 5);

        // Test filtering by family
        let filter = AssetFilter {
            family: Some("VIDEO".to_string()),
            ..Default::default()
        };
        let video_assets: Vec<AssetSummaryDto> = handler
            .list_paginated(
                filter,
                PageParams {
                    page: 1,
                    page_size: 10,
                },
            )
            .await
            .expect("Failed to filter");
        assert_eq!(video_assets.len(), 5);
        assert!(video_assets.iter().all(|a| a.family == "VIDEO"));

        // Test search query
        let filter_search = AssetFilter {
            search_query: Some("file-1".to_string()),
            ..Default::default()
        };
        let searched: Vec<AssetSummaryDto> = handler
            .list_paginated(filter_search, PageParams::default())
            .await
            .expect("Failed search");
        // Should find file-1 and file-10
        assert_eq!(searched.len(), 2);
    }
}
