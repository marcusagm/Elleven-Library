use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, PageParams};
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
                CAST(NULL AS REAL) as duration_secs,
                CAST(NULL AS TEXT) as "technical_payload: serde_json::Value",
                CAST(NULL AS TEXT) as "semantic_payload: serde_json::Value"
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
                m.width as "width: i32", m.height as "height: i32", m.duration_secs,
                m.technical_payload as "technical_payload: serde_json::Value",
                m.semantic_payload as "semantic_payload: serde_json::Value"
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
            "SELECT id, name, state, format_type, family, created_at FROM v2_assets WHERE 1=1 ",
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
