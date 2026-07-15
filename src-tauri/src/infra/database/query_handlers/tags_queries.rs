use crate::core::error::AppResult;
use crate::core::models::Tag;
use sqlx::SqlitePool;

/// Lists tags in the database.
///
/// # Returns
///
/// * `Ok(Vec<Tag>)` if the tags were found successfully.
/// * `Err(sqlx::Error)` if the tags could not be found.
pub async fn list_tags(pool: &SqlitePool, _registry: &crate::core::formats::registry::FormatRegistry) -> AppResult<Vec<crate::core::models::Tag>> {
    let rows = sqlx::query_as!(
        crate::infra::database::models::TagDb,
        r#"SELECT id as "id!", name as "name!", color, parent_id, order_index as "order_index!" FROM tags ORDER BY order_index ASC, name ASC"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Tag::from).collect())
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
pub async fn get_tags_for_asset(pool: &SqlitePool, _registry: &crate::core::formats::registry::FormatRegistry, asset_id: &str) -> AppResult<Vec<crate::core::models::Tag>> {
    let rows = sqlx::query_as!(
        crate::infra::database::models::TagDb,
        r#"SELECT t.id as "id!", t.name as "name!", t.color, t.parent_id, t.order_index as "order_index!"
           FROM tags t
           JOIN asset_tags at ON t.id = at.tag_id
           WHERE at.asset_id = ?
           ORDER BY t.order_index ASC, t.name ASC"#,
        asset_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Tag::from).collect())
}

