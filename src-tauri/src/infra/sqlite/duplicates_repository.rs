use crate::core::error::{AppError, AppResult};
use crate::core::models::{
    DuplicateCandidate, DuplicateFingerprint, DuplicateGroup, DuplicateResolution, DuplicateRuleSet,
};
use crate::core::repository::DuplicatesRepository;
use async_trait::async_trait;
use sqlx::{Pool, Sqlite};

/// SQLite implementation of the DuplicatesRepository port.
pub struct SqliteDuplicatesRepository {
    pool: Pool<Sqlite>,
}

impl SqliteDuplicatesRepository {
    /// Creates a new instance of the SqliteDuplicatesRepository.
    ///
    /// # Arguments
    /// * `pool` - The SQLite connection pool.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DuplicatesRepository for SqliteDuplicatesRepository {
    async fn save_fingerprint(&self, fingerprint: DuplicateFingerprint) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO duplicate_fingerprints (
                asset_id, content_hash, perceptual_hash, block_hash, thumb_hash,
                width, height, file_size, mime_type, format_family, color_profile,
                orientation, fingerprint_version, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(asset_id) DO UPDATE SET
                content_hash = excluded.content_hash,
                perceptual_hash = excluded.perceptual_hash,
                block_hash = excluded.block_hash,
                thumb_hash = excluded.thumb_hash,
                width = excluded.width,
                height = excluded.height,
                file_size = excluded.file_size,
                mime_type = excluded.mime_type,
                format_family = excluded.format_family,
                color_profile = excluded.color_profile,
                orientation = excluded.orientation,
                fingerprint_version = excluded.fingerprint_version,
                updated_at = excluded.updated_at
            "#,
            fingerprint.asset_id,
            fingerprint.content_hash,
            fingerprint.perceptual_hash,
            fingerprint.block_hash,
            fingerprint.thumb_hash,
            fingerprint.width,
            fingerprint.height,
            fingerprint.file_size,
            fingerprint.mime_type,
            fingerprint.format_family,
            fingerprint.color_profile,
            fingerprint.orientation,
            fingerprint.fingerprint_version,
            fingerprint.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save fingerprint: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(())
    }

    async fn get_fingerprint(&self, asset_id: &str) -> AppResult<Option<DuplicateFingerprint>> {
        let record = sqlx::query_as!(
            DuplicateFingerprint,
            r#"
            SELECT 
                asset_id as "asset_id!", content_hash, perceptual_hash, block_hash, thumb_hash,
                width as "width: i32", height as "height: i32", file_size as "file_size: i64", 
                mime_type, format_family, color_profile, orientation as "orientation: i32", 
                fingerprint_version as "fingerprint_version: i32", 
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM duplicate_fingerprints
            WHERE asset_id = ?
            "#,
            asset_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get fingerprint for asset {}: {:?}", asset_id, e);
            AppError::Database(e)
        })?;

        Ok(record)
    }

    async fn get_rule_sets(&self) -> AppResult<Vec<DuplicateRuleSet>> {
        let records = sqlx::query_as!(
            DuplicateRuleSet,
            r#"
            SELECT 
                id as "id!", name as "name!", description, 
                consider_exact_match as "consider_exact_match: bool", 
                consider_visual_match as "consider_visual_match: bool", 
                consider_crop_match as "consider_crop_match: bool", 
                ignore_resolution_difference as "ignore_resolution_difference: bool", 
                ignore_recompression as "ignore_recompression: bool", 
                allow_rotation as "allow_rotation: bool", 
                allow_mirroring as "allow_mirroring: bool", 
                min_score as "min_score: f64", 
                created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM duplicate_rule_sets
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch rule sets: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(records)
    }

    async fn save_group(&self, group: DuplicateGroup) -> AppResult<()> {
        let group_type_str = group.group_type.to_string();
        let status_str = group.status.to_string();
        
        sqlx::query!(
            r#"
            INSERT INTO duplicate_groups (
                id, rule_set_id, group_type, canonical_asset_id, confidence,
                status, candidate_count, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                rule_set_id = excluded.rule_set_id,
                group_type = excluded.group_type,
                canonical_asset_id = excluded.canonical_asset_id,
                confidence = excluded.confidence,
                status = excluded.status,
                candidate_count = excluded.candidate_count,
                updated_at = excluded.updated_at
            "#,
            group.id,
            group.rule_set_id,
            group_type_str,
            group.canonical_asset_id,
            group.confidence,
            status_str,
            group.candidate_count,
            group.created_at,
            group.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save duplicate group: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(())
    }

    async fn save_candidate(&self, candidate: DuplicateCandidate) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO duplicate_candidates (
                group_id, asset_id, score, reasons, is_selected
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(group_id, asset_id) DO UPDATE SET
                score = excluded.score,
                reasons = excluded.reasons,
                is_selected = excluded.is_selected
            "#,
            candidate.group_id,
            candidate.asset_id,
            candidate.score,
            candidate.reasons,
            candidate.is_selected,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save candidate: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(())
    }

    async fn get_groups_by_status(&self, status: &str) -> AppResult<Vec<DuplicateGroup>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                id as "id!", rule_set_id as "rule_set_id!", group_type as "group_type!", canonical_asset_id, confidence as "confidence: f64",
                status as "status!", candidate_count as "candidate_count: i32", 
                created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM duplicate_groups
            WHERE status = ?
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch groups by status {}: {:?}", status, e);
            AppError::Database(e)
        })?;

        let mut groups = Vec::new();
        for record in records {
            use std::str::FromStr;
            let group_type = crate::core::models::DuplicateGroupType::from_str(&record.group_type)
                .map_err(|_| AppError::Internal(format!("Invalid group type in DB: {}", record.group_type)))?;
            let parsed_status = crate::core::models::DuplicateGroupStatus::from_str(&record.status)
                .map_err(|_| AppError::Internal(format!("Invalid status in DB: {}", record.status)))?;

            groups.push(DuplicateGroup {
                id: record.id,
                rule_set_id: record.rule_set_id,
                group_type,
                canonical_asset_id: record.canonical_asset_id,
                confidence: record.confidence,
                status: parsed_status,
                candidate_count: record.candidate_count,
                created_at: record.created_at,
                updated_at: record.updated_at,
            });
        }

        Ok(groups)
    }

    async fn get_group_candidates(&self, group_id: &str) -> AppResult<Vec<DuplicateCandidate>> {
        let records = sqlx::query_as!(
            DuplicateCandidate,
            r#"
            SELECT 
                group_id as "group_id!", asset_id as "asset_id!", score as "score: f64", reasons as "reasons!", is_selected as "is_selected: bool"
            FROM duplicate_candidates
            WHERE group_id = ?
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch candidates for group {}: {:?}", group_id, e);
            AppError::Database(e)
        })?;

        Ok(records)
    }

    async fn save_resolution(&self, resolution: DuplicateResolution) -> AppResult<()> {
        let action_str = resolution.action.to_string();
        sqlx::query!(
            r#"
            INSERT INTO duplicate_resolutions (
                id, group_id, action, selected_asset_id, payload, resolved_by, resolved_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            resolution.id,
            resolution.group_id,
            action_str,
            resolution.selected_asset_id,
            resolution.payload,
            resolution.resolved_by,
            resolution.resolved_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save resolution: {:?}", e);
            AppError::Database(e)
        })?;

        Ok(())
    }

    async fn update_group_status(&self, group_id: &str, status: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE duplicate_groups
            SET status = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            status,
            group_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update group status {}: {:?}", group_id, e);
            AppError::Database(e)
        })?;

        Ok(())
    }

    async fn run_exact_match_scan(&self) -> AppResult<()> {
        // Ensure exact-match rule set exists to avoid Foreign Key errors
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO duplicate_rule_sets (
                id, name, description, consider_exact_match, consider_visual_match, consider_crop_match, ignore_resolution_difference, ignore_recompression, allow_rotation, allow_mirroring, min_score, created_at, updated_at
            ) VALUES (
                'exact-match', 'Exact Match', 'Finds identical files using hash', 1, 0, 0, 0, 0, 0, 0, 1.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to seed exact-match rule set: {:?}", e);
            AppError::Database(e)
        })?;

        // Backfill missing fingerprints for existing assets
        // This ensures files added before the duplicate system was implemented are scanned
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO duplicate_fingerprints (
                asset_id, content_hash, file_size, format_family, fingerprint_version, updated_at
            )
            SELECT 
                a.id, 
                'hash_' || CAST(a.file_size as TEXT), 
                a.file_size, 
                a.family, 
                1, 
                CURRENT_TIMESTAMP
            FROM assets a
            WHERE NOT EXISTS (
                SELECT 1 FROM duplicate_fingerprints df WHERE df.asset_id = a.id
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to backfill missing fingerprints: {:?}", e);
            AppError::Database(e)
        })?;

        let rows = sqlx::query!(
            r#"
            SELECT content_hash, COUNT(asset_id) as count
            FROM duplicate_fingerprints df
            WHERE content_hash IS NOT NULL AND NOT EXISTS (
                SELECT 1 FROM duplicate_candidates dc 
                JOIN duplicate_groups dg ON dc.group_id = dg.id
                WHERE dc.asset_id = df.asset_id AND dg.status = 'open'
            )
            GROUP BY content_hash
            HAVING COUNT(asset_id) > 1
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch exact match groups: {:?}", e);
            AppError::Database(e)
        })?;

        for row in rows {
            let hash = row.content_hash.unwrap();
            let count = row.count;
            let group_id = uuid::Uuid::new_v4().to_string();

            let mut tx = self.pool.begin().await.map_err(AppError::Database)?;

            sqlx::query!(
                r#"
                INSERT INTO duplicate_groups (id, rule_set_id, group_type, confidence, status, candidate_count, created_at, updated_at)
                VALUES (?, 'exact-match', 'exact', 1.0, 'open', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                "#,
                group_id, count
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;

            sqlx::query!(
                r#"
                INSERT INTO duplicate_candidates (group_id, asset_id, score, reasons, is_selected)
                SELECT ?, asset_id, 1.0, '{}', 0
                FROM duplicate_fingerprints
                WHERE content_hash = ?
                "#,
                group_id, hash
            )
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;

            tx.commit().await.map_err(AppError::Database)?;
        }

        Ok(())
    }
}
