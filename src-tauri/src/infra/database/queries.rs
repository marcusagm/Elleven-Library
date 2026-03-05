use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetState};
use crate::core::repository::AssetQueryHandler;
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};

/// SQLite implementation of the AssetQueryHandler port.
pub struct SqliteAssetQueries {
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
            crate::infra::database::models::AssetDb,
            r#"SELECT id as "id!", name as "name!", path as "path!", state as "state!", format_type as "format_type!", family as "family!", file_size as "file_size!", created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>" FROM v2_assets"#
        )
        .fetch_all(&self.pool)
        .await?;

        // Map DB DTO to Domain Entity
        let assets = rows
            .into_iter()
            .map(|row| Asset {
                id: row.id,
                name: row.name,
                path: PathBuf::from(row.path),
                state: AssetState::from_str(&row.state).unwrap_or(AssetState::Unknown),
                format_type: row.format_type,
                family: row.family,
                file_size: row.file_size as u64,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(assets)
    }
}

/// Tests for the SqliteAssetQueries struct.
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Tests the asset insertion and query functionality.
    #[tokio::test]
    async fn test_asset_insertion_and_query() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory DB");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Insert mock data
        sqlx::query("INSERT INTO v2_assets (id, name, path, state, format_type, family, file_size, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind("test-id-123")
            .bind("Logo_Temp.png")
            .bind("/tmp/Logo_Temp.png")
            .bind("Idle")
            .bind("image/png")
            .bind("IMAGE")
            .bind(1024)
            .bind(Utc::now())
            .execute(&pool)
            .await
            .expect("Failed to insert mock asset");

        let handler = SqliteAssetQueries::new(pool);
        let assets = handler.find_all().await.expect("Failed to find assets");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Logo_Temp.png");
        assert_eq!(assets[0].state, AssetState::Idle);
    }
}
