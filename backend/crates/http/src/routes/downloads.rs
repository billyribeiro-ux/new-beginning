//! `/v1/downloads/*` — entitlement-checked presigned downloads.
//!
//! BACKEND.md §11 (downloads catalog + grants), §22 rule 8 (audit + state
//! mutation in one tx).
//!
//! Two endpoints:
//!
//! * `GET /v1/downloads` — every catalog row the user is entitled to
//!   (joined via active licenses). Read-only; no audit row.
//! * `GET /v1/downloads/{download_id}/url` — verifies entitlement, then
//!   in **one transaction**: bumps the `download_grants` counter +
//!   writes an audit row + signs a short-TTL presigned GET. The audit +
//!   grant bump MUST live in the same tx as the URL mint (the mint
//!   itself doesn't touch the DB, but everything we record about it does).

use axum::extract::{Path, State};
use axum::Json;
use common::error::AppError;
use serde::Serialize;
use std::time::Duration;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::AuthSession;
use crate::state::AppState;

/// Presign TTL — short blast radius for leaked URLs, comfortable for a
/// click-to-download. Mirrored to the BFF in the response.
const DOWNLOAD_URL_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Serialize)]
pub struct DownloadRow {
    pub id: String,
    pub product_id: String,
    pub platform: String,
    pub version: String,
    pub sha256: String,
    pub size_bytes: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub released_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub downloads: Vec<DownloadRow>,
}

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ListResponse>, AppError> {
    let catalog = state
        .downloads_catalog
        .list_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let downloads = catalog
        .into_iter()
        .map(|c| DownloadRow {
            id: c.id.to_string(),
            product_id: c.product_id.to_string(),
            platform: c.platform,
            version: c.version,
            sha256: c.sha256,
            size_bytes: c.size_bytes,
            released_at: c.released_at,
        })
        .collect();
    Ok(Json(ListResponse { downloads }))
}

#[derive(Debug, Serialize)]
pub struct UrlResponse {
    pub url: String,
    pub ttl_seconds: u64,
    pub download_count: i32,
}

pub async fn url(
    State(state): State<AppState>,
    session: AuthSession,
    Path(download_id): Path<Uuid>,
) -> Result<Json<UrlResponse>, AppError> {
    // Resolve the catalog row.
    let entry = state
        .downloads_catalog
        .find_by_id(download_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    // Entitlement: caller must hold an active license for the product
    // this catalog row belongs to.
    let licenses = state
        .licenses
        .list_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !licenses.iter().any(|l| l.product_id == entry.product_id) {
        return Err(AppError::Forbidden);
    }

    // Open ONE transaction for the grant bump + audit row. The presign
    // doesn't hit the DB, but the side effects we record about it (the
    // grant counter + audit log) live atomically. BACKEND.md §22 rule 8.
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let grant = state
        .download_grants
        .record_access_in_tx(&mut tx, session.user_id, entry.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    state
        .audit
        .record_in_tx(
            &mut tx,
            Some(session.user_id),
            "download.url_minted",
            "downloads_catalog",
            &entry.id.to_string(),
            serde_json::json!({
                "platform": entry.platform,
                "version": entry.version,
                "product_id": entry.product_id.to_string(),
                "grant_count": grant.download_count,
            }),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let url = state
        .r2
        .presigned_get(&entry.file_r2_key, DOWNLOAD_URL_TTL)
        .map_err(|e| AppError::External {
            service: "r2",
            source: anyhow::anyhow!(e),
        })?;

    Ok(Json(UrlResponse {
        url: url.to_string(),
        ttl_seconds: DOWNLOAD_URL_TTL.as_secs(),
        download_count: grant.download_count,
    }))
}
