use std::sync::Arc;
use tauri::State;
use tracing::instrument;

use crate::core::error::AppResult;
use crate::core::workflows::thumbnails::priority::ThumbnailPriorityState;

/// Commands for thumbnail priority management.
///
/// Under Hexagonal Architecture, this is the "Delivery Layer" (Infrastructure/Application layer)
/// that translates UI requests into domain/workflow actions.
///
/// Prioritizes a batch of asset IDs for thumbnail generation.
/// This pushes the IDs to the LIFO queue in the shared `ThumbnailPriorityState`.
#[tauri::command]
#[instrument(skip(priority_state))]
pub async fn set_thumbnail_priority(
    ids: Vec<String>,
    priority_state: State<'_, Arc<ThumbnailPriorityState>>,
) -> AppResult<()> {
    priority_state.push_priorities(ids);
    Ok(())
}
