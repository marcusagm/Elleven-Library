use crate::core::error::AppResult;
use crate::core::models::{DuplicateCandidate, DuplicateGroup, DuplicateResolutionAction};
use crate::feature::duplicates::commands::DuplicateCommandService;
use crate::feature::duplicates::queries::DuplicateQueryService;
use tauri::State;

/// Retrieves duplicate groups by status.
#[tauri::command]
pub async fn get_duplicate_groups(
    status: String,
    query_service: State<'_, DuplicateQueryService>,
) -> AppResult<Vec<DuplicateGroup>> {
    query_service.get_groups_by_status(&status).await
}

/// Retrieves candidates for a duplicate group.
#[tauri::command]
pub async fn get_duplicate_candidates(
    group_id: String,
    query_service: State<'_, DuplicateQueryService>,
) -> AppResult<Vec<DuplicateCandidate>> {
    query_service.get_group_candidates(&group_id).await
}

/// Resolves a duplicate group with a given action.
#[tauri::command]
pub async fn resolve_duplicate_group(
    group_id: String,
    action: String,
    kept_asset_ids: Option<Vec<String>>,
    command_service: State<'_, DuplicateCommandService>,
) -> AppResult<()> {
    use std::str::FromStr;
    let parsed_action = DuplicateResolutionAction::from_str(&action)
        .map_err(|_| crate::core::error::AppError::ValidationFailed(format!("Invalid action: {}", action)))?;

    command_service
        .resolve_group(&group_id, parsed_action, kept_asset_ids)
        .await
}

/// Dispara uma varredura por duplicados no repositório.
#[tauri::command]
pub async fn start_duplicate_scan(
    command_service: State<'_, DuplicateCommandService>,
) -> AppResult<()> {
    command_service.start_duplicate_scan().await?;
    Ok(())
}
