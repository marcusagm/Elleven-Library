use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag};
use crate::core::repository::AssetQueryHandler;
use crate::infra::database::models::{AssetDb, AssetSummaryDb};
use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use chrono::{DateTime, Utc};

/// SQLite implementation of the AssetQueryHandler port.
pub struct SqliteAssetQueries {
    /// The database connection pool.
    pool: SqlitePool,
}

/// Implementation of the SqliteAssetQueries struct.
impl SqliteAssetQueries {
    /// Creates a new instance with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - The connection pool to use.
    ///
    /// # Returns
    ///
    /// * `SqliteAssetQueries` - The new instance.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Implementation of the AssetQueryHandler port for SqliteAssetQueries.
#[async_trait]
impl AssetQueryHandler for SqliteAssetQueries {
    /// Finds all assets in the database.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Asset>)` if the assets were found successfully.
    /// * `Err(sqlx::Error)` if the assets could not be found.
    async fn find_all(&self) -> AppResult<Vec<Asset>> {
        let rows = sqlx::query_as!(
            AssetDb,
            r#"
            SELECT
                id as "id!", name as "name!", path as "path!", state as "state!",
                format_type as "format_type!", family as "family!", file_size as "file_size!",
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>",
                CAST(NULL AS INTEGER) as "width: i32",
                CAST(NULL AS INTEGER) as "height: i32",
                CAST(NULL AS REAL) as "duration_secs: f64",
                CAST(NULL AS TEXT) as "technical_payload: serde_json::Value",
                CAST(NULL AS TEXT) as "semantic_payload: serde_json::Value",
                CAST(NULL AS TEXT) as "dominant_color: serde_json::Value",
                folder_id as "folder_id?",
                thumbnail_path as "thumbnail_path?"
            FROM v2_assets
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Asset::from).collect::<Vec<Asset>>())
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
        let row = sqlx::query_as!(
            AssetDb,
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                m.width as "width: i32", m.height as "height: i32", m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                m.dominant_colors as "dominant_color: serde_json::Value",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?"
            FROM v2_assets a
            LEFT JOIN v2_asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Asset::from))
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
            "SELECT id, name, state, format_type, family, created_at, folder_id FROM v2_assets WHERE 1=1 ",
        );

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
            query_builder.push(" AND folder_id = ");
            query_builder.push_bind(folder_id);
        }

        if let Some(tags) = filter.tags {
            if !tags.is_empty() {
                query_builder.push(" AND id IN (SELECT asset_id FROM v2_asset_tags WHERE tag_id IN (SELECT id FROM v2_tags WHERE name IN (");
                let mut first = true;
                for tag in tags {
                    if !first {
                        query_builder.push(", ");
                    }
                    query_builder.push_bind(tag);
                    first = false;
                }
                query_builder.push(")))");
            }
        }

        if let Some(untagged) = filter.untagged {
            if untagged {
                query_builder.push(" AND id NOT IN (SELECT asset_id FROM v2_asset_tags)");
            }
        }

        // Ordering as per Sprint decision: created_at DESC, name ASC
        query_builder.push(" ORDER BY created_at DESC, name ASC ");

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
                r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM v2_folders WHERE parent_id = ?"#,
                parent
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                crate::infra::database::models::FolderDb,
                r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM v2_folders WHERE parent_id IS NULL"#
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
            r#"SELECT id as "id!", parent_id, name as "name!", path as "path!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>" FROM v2_folders WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Folder::from))
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
            r#"SELECT id as "id!", name as "name!", color, parent_id FROM v2_tags ORDER BY name ASC"#
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
                a.id, a.name, a.state, a.format_type, a.family, a.created_at, a.folder_id
            FROM v2_assets a
            LEFT JOIN v2_asset_metadata_envelope m ON a.id = m.asset_id
            WHERE 1=1 AND
            "#,
        );

        build_search_where_clause(&criteria.root_group, &mut query_builder);

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
            r#"SELECT id as "id!" FROM v2_assets WHERE thumbnail_path IS NULL LIMIT ?"#,
            limit_i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect::<Vec<String>>())
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
        let row = sqlx::query_as!(
            AssetDb,
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                m.width as "width: i32", m.height as "height: i32", m.duration_secs as "duration_secs: f64",
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value",
                m.dominant_colors as "dominant_color: serde_json::Value",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?"
            FROM v2_assets a
            LEFT JOIN v2_asset_metadata_envelope m ON a.id = m.asset_id
            WHERE a.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::core::error::AppError::NotFound(id.to_string()))?;

        Ok(row.into())
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
            r#"SELECT path as "path!", file_size as "file_size!", updated_at as "updated_at!: DateTime<Utc>" FROM v2_assets WHERE path LIKE ?"#,
            pattern
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.path, (r.file_size, r.updated_at)))
            .collect())
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
            sqlx::query("INSERT INTO v2_assets (id, name, path, state, format_type, family, file_size, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
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

        let handler = SqliteAssetQueries::new(pool);

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
