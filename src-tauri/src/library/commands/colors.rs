//! Tauri commands for color palette operations.
//!
//! Provides endpoints for retrieving extracted colors, re-extracting
//! colors for individual assets or the entire library.

use crate::core::error::AppResult;
use crate::db::models::AssetColor;
use crate::db::Db;
use std::sync::Arc;
use tauri::State;
use tracing::{error, info, warn};

/// Retrieves the extracted color palette for a specific asset.
///
/// Returns an empty vector if no colors have been extracted yet.
///
/// # Errors
///
/// Returns `AppError` if the database query fails.
#[tauri::command]
pub async fn get_asset_colors(db: State<'_, Arc<Db>>, asset_id: i64) -> AppResult<Vec<AssetColor>> {
    Ok(db.get_asset_colors(asset_id).await?)
}

/// Re-extracts the color palette for a single asset from its thumbnail.
///
/// Deletes existing colors and performs a fresh k-means analysis.
///
/// # Errors
///
/// Returns `AppError` if the asset is not found, has no thumbnail,
/// or if the color extraction algorithm fails.
#[tauri::command]
pub async fn reextract_asset_colors(
    db: State<'_, Arc<Db>>,
    asset_id: i64,
) -> AppResult<Vec<AssetColor>> {
    let asset_row: Option<(String, String)> = sqlx::query_as(
        "SELECT thumbnail_path, media_type FROM assets WHERE id = ? AND thumbnail_path IS NOT NULL",
    )
    .bind(asset_id)
    .fetch_optional(&db.pool)
    .await?;

    let (thumbnail_path, media_type) = match asset_row {
        Some(row) => row,
        None => {
            warn!(
                "COLOR: Cannot re-extract colors for asset {} — no thumbnail or not found",
                asset_id
            );
            return Ok(Vec::new());
        }
    };

    if media_type != "Image" {
        warn!(
            "COLOR: Skipping re-extraction for asset {} — media_type is '{}', not 'Image'",
            asset_id, media_type
        );
        return Ok(Vec::new());
    }

    let thumbnail_file_path = std::path::Path::new(&thumbnail_path);
    if !thumbnail_file_path.exists() {
        warn!(
            "COLOR: Thumbnail file does not exist for asset {}: {:?}",
            asset_id, thumbnail_path
        );
        return Ok(Vec::new());
    }

    match crate::thumbnails::color_analysis::extract_color_palette(thumbnail_file_path, None) {
        Ok(extracted_colors) => {
            let asset_colors: Vec<AssetColor> = extracted_colors
                .iter()
                .enumerate()
                .map(|(index, color)| AssetColor {
                    id: 0,
                    asset_id: asset_id.to_string(),
                    hex_color: color.hex_value.clone(),
                    lab_lightness: color.lab_lightness,
                    lab_green_red: color.lab_green_red,
                    lab_blue_yellow: color.lab_blue_yellow,
                    percentage: color.percentage,
                    rank: (index + 1) as i32,
                })
                .collect();

            db.insert_asset_colors(asset_id, &asset_colors).await?;

            if let Some(dominant) = extracted_colors.first() {
                db.update_dominant_color(asset_id, &dominant.hex_value)
                    .await?;
            }

            info!(
                "COLOR: Re-extracted {} colors for asset {}",
                asset_colors.len(),
                asset_id
            );

            Ok(db.get_asset_colors(asset_id).await?)
        }
        Err(extraction_error) => {
            error!(
                "COLOR: Failed to re-extract colors for asset {}: {}",
                asset_id, extraction_error
            );
            Ok(Vec::new())
        }
    }
}

/// Re-extracts color palettes for all image assets in the library.
///
/// This command processes assets in batches, clearing existing colors
/// and performing fresh k-means analysis on each thumbnail.
///
/// # Returns
///
/// The number of assets successfully processed.
///
/// # Errors
///
/// Returns `AppError` if the database query fails. Individual asset
/// extraction errors are logged but do not halt the batch.
#[tauri::command]
pub async fn reextract_all_colors(db: State<'_, Arc<Db>>) -> AppResult<i64> {
    info!("COLOR: Starting full library color re-extraction");

    let all_image_assets: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, thumbnail_path FROM assets
         WHERE media_type = 'Image' AND thumbnail_path IS NOT NULL",
    )
    .fetch_all(&db.pool)
    .await?;

    let total_asset_count = all_image_assets.len();
    let mut successfully_processed_count: i64 = 0;

    for (asset_id, thumbnail_path) in &all_image_assets {
        let thumbnail_file_path = std::path::Path::new(thumbnail_path);
        if !thumbnail_file_path.exists() {
            continue;
        }

        match crate::thumbnails::color_analysis::extract_color_palette(thumbnail_file_path, None) {
            Ok(extracted_colors) => {
                let asset_colors: Vec<AssetColor> = extracted_colors
                    .iter()
                    .enumerate()
                    .map(|(index, color)| AssetColor {
                        id: 0,
                        asset_id: asset_id.to_string(),
                        hex_color: color.hex_value.clone(),
                        lab_lightness: color.lab_lightness,
                        lab_green_red: color.lab_green_red,
                        lab_blue_yellow: color.lab_blue_yellow,
                        percentage: color.percentage,
                        rank: (index + 1) as i32,
                    })
                    .collect();

                if let Err(db_error) = db.insert_asset_colors(*asset_id, &asset_colors).await {
                    error!(
                        "COLOR: DB error saving colors for asset {}: {}",
                        asset_id, db_error
                    );
                    continue;
                }

                if let Some(dominant) = extracted_colors.first() {
                    let _ = db
                        .update_dominant_color(*asset_id, &dominant.hex_value)
                        .await;
                }

                successfully_processed_count += 1;
            }
            Err(extraction_error) => {
                warn!(
                    "COLOR: Skipped asset {} — extraction failed: {}",
                    asset_id, extraction_error
                );
            }
        }
    }

    info!(
        "COLOR: Full re-extraction complete — {}/{} assets processed",
        successfully_processed_count, total_asset_count
    );

    Ok(successfully_processed_count)
}
