//! Database abstraction layer for Mundam.
//!
//! This module handles the connection pool, schema initialization, and
//! provides a central entry point for all database operations.

pub mod assets;
pub mod folders;
pub mod models;
pub mod search;
pub mod settings;
pub mod smart_folders;
pub mod tags;

use crate::error::AppResult;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

/// The main database handle, wrapping a SQLite connection pool.
///
/// This struct is shared across the application via Tauri's state management.
pub struct Db {
    /// The underlying SQLite connection pool.
    pub pool: SqlitePool,
}

impl Db {
    /// Creates a new database instance or opens an existing one.
    ///
    /// This function also initializes the database with the required schema
    /// and runs any pending migrations.
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the SQLite database file should be located.
    ///
    /// # Errors
    ///
    /// Returns a `sqlx::Error` if the connection fails or if migrations fail to run.
    pub async fn new(path: PathBuf) -> AppResult<Self> {
        use sqlx::sqlite::{
            SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
        };
        use std::str::FromStr;
        use std::time::Duration;

        let url = format!("sqlite:{}", path.to_string_lossy());
        let options = SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(30));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Initialize schema and run migrations from the /migrations directory
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Returns a reference to the underlying connection pool.
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// Performs routine database maintenance missions.
    ///
    /// Runs `VACUUM` to reclaim space and `ANALYZE` to update query planner statistics.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the maintenance queries fail.
    pub async fn run_maintenance(&self) -> AppResult<()> {
        println!("DEBUG: DB - Running Maintenance (VACUUM + ANALYZE)");
        sqlx::query("VACUUM").execute(&self.pool).await?;
        sqlx::query("ANALYZE").execute(&self.pool).await?;
        Ok(())
    }
}
