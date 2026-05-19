//! Clip Studio Paint (.clip) preview extractor.
//!
//! Ported from V1 backend.

use byteorder::{BigEndian, ReadBytesExt};
use sqlx::sqlite::SqlitePoolOptions;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use thiserror::Error;

const CLIP_MAGIC: &[u8; 8] = b"CSFCHUNK";
const SQL_CHUNK_NAME: &[u8; 8] = b"CHNKSQLi";
const FOOTER_CHUNK_NAME: &[u8; 8] = b"CHNKFoot";

#[derive(Debug, Error)]
pub enum ClipError {
    #[error("Invalid CLIP format: missing magic")]
    InvalidFormat,
    #[error("CLIP missing SQLite chunk")]
    MissingSqlChunk,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub fn extract_clip_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != CLIP_MAGIC { return Err(ClipError::InvalidFormat.into()); }
    file.seek(SeekFrom::Current(16))?; // Skip length and offsets

    let mut sql_data = None;
    loop {
        let mut name = [0u8; 8];
        if file.read_exact(&mut name).is_err() { break; }
        let len = file.read_u64::<BigEndian>()?;
        let start = file.stream_position()?;
        if &name == SQL_CHUNK_NAME {
            let mut data = vec![0u8; len as usize];
            file.read_exact(&mut data)?;
            sql_data = Some(data);
            break;
        }
        if &name == FOOTER_CHUNK_NAME { break; }
        file.seek(SeekFrom::Start(start + len))?;
    }

    let db_bytes = sql_data.ok_or(ClipError::MissingSqlChunk)?;
    let temp_dir = std::env::temp_dir();
    let temp_db = temp_dir.join(format!("mundam_clip_{}.sqlite", uuid::Uuid::new_v4()));
    std::fs::write(&temp_db, db_bytes)?;

    let result = tauri::async_runtime::block_on(async {
        query_preview(&temp_db).await
    });

    let _ = std::fs::remove_file(&temp_db);
    result.map_err(|e| e.into())
}

pub fn extract_clip_metadata(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != CLIP_MAGIC { return Err(ClipError::InvalidFormat.into()); }
    file.seek(SeekFrom::Current(16))?; // Skip length and offsets

    let mut sql_data = None;
    loop {
        let mut name = [0u8; 8];
        if file.read_exact(&mut name).is_err() { break; }
        let len = file.read_u64::<BigEndian>()?;
        let start = file.stream_position()?;
        if &name == SQL_CHUNK_NAME {
            let mut data = vec![0u8; len as usize];
            file.read_exact(&mut data)?;
            sql_data = Some(data);
            break;
        }
        if &name == FOOTER_CHUNK_NAME { break; }
        file.seek(SeekFrom::Start(start + len))?;
    }

    let db_bytes = sql_data.ok_or(ClipError::MissingSqlChunk)?;
    let temp_dir = std::env::temp_dir();
    let temp_db = temp_dir.join(format!("mundam_clip_meta_{}.sqlite", uuid::Uuid::new_v4()));
    std::fs::write(&temp_db, db_bytes)?;

    let result = tauri::async_runtime::block_on(async {
        query_metadata(&temp_db).await
    });

    let _ = std::fs::remove_file(&temp_db);
    result.map_err(|e| e.into())
}

async fn query_preview(path: &Path) -> Result<(Vec<u8>, String), ClipError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}", path.to_str().unwrap_or_default()))
        .await?;
    let (data,): (Vec<u8>,) = sqlx::query_as("SELECT ImageData FROM CanvasPreview LIMIT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| ClipError::DatabaseError(e.to_string()))?;
    pool.close().await;
    Ok((data, "image/png".to_string()))
}

async fn query_metadata(path: &Path) -> Result<serde_json::Value, ClipError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}", path.to_str().unwrap_or_default()))
        .await?;
        
    let row: (f64, f64, f64) = sqlx::query_as("SELECT CanvasWidth, CanvasHeight, CanvasResolution FROM Canvas LIMIT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| ClipError::DatabaseError(e.to_string()))?;
        
    pool.close().await;
    
    let mut technical = serde_json::json!({
        "container": "Clip Studio Paint",
        "metadata_support": "Full"
    });
    
    technical["width"] = serde_json::json!(row.0 as u32);
    technical["height"] = serde_json::json!(row.1 as u32);
    technical["dpi"] = serde_json::json!(row.2 as u32);
    technical["metadata_source"] = serde_json::json!("sqlite");

    Ok(serde_json::json!({
        "technical": technical,
        "semantic": {}
    }))
}
