//! `/v1/indicators/*` — owned licenses + downloads per product.
//!
//! BACKEND.md §11. The user's dashboard shows one row per indicator
//! product they own, with the license-key prefix (never the plaintext —
//! that ships once via email at issuance time) and a "download" affordance
//! that resolves to a presigned R2 URL via `/v1/downloads/{id}/url`.
//!
//! Three endpoints:
//!
//! * `GET /v1/indicators` — every active license + its product.
//! * `GET /v1/indicators/{slug}/key` — the license-key **prefix** only.
//!   The plaintext exists exactly once at issuance (in the Stripe
//!   webhook's response, delivered via email — see PR #13's mailer).
//!   This endpoint is the dashboard's "what was my key prefix?" lookup.
//! * `GET /v1/indicators/{slug}/downloads` — catalog rows the user is
//!   entitled to for this product (across platforms / versions).

use axum::extract::{Path, State};
use axum::Json;
use common::error::AppError;
use serde::Serialize;
use time::OffsetDateTime;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct IndicatorRow {
    pub product_id: String,
    pub product_slug: String,
    pub product_name: String,
    pub license_key_prefix: String,
    pub source: String,
    #[serde(with = "time::serde::rfc3339")]
    pub issued_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub indicators: Vec<IndicatorRow>,
}

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ListResponse>, AppError> {
    let licenses = state
        .licenses
        .list_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    // Resolve product slug + name per license. Small N (handful), one
    // query each is fine; a future optimization can JOIN in the repo.
    let mut rows = Vec::with_capacity(licenses.len());
    for lic in licenses {
        let product = state
            .products
            .find_by_id(lic.product_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let Some(product) = product else {
            // Orphaned license — log + skip rather than 500.
            tracing::warn!(
                license_id = %lic.id,
                product_id = %lic.product_id,
                "license references a deleted product; hiding from indicators list",
            );
            continue;
        };
        rows.push(IndicatorRow {
            product_id: lic.product_id.to_string(),
            product_slug: product.slug,
            product_name: product.name,
            license_key_prefix: lic.license_key_prefix,
            source: lic.source,
            issued_at: lic.issued_at,
        });
    }
    Ok(Json(ListResponse { indicators: rows }))
}

#[derive(Debug, Serialize)]
pub struct KeyResponse {
    /// Always the prefix; the plaintext is only delivered once at
    /// issuance via email (BACKEND.md §8.5). If the user lost it they go
    /// through admin re-issuance — never re-derivable from the DB.
    pub license_key_prefix: String,
}

pub async fn key_for_slug(
    State(state): State<AppState>,
    session: AuthSession,
    Path(slug): Path<String>,
) -> Result<Json<KeyResponse>, AppError> {
    let product = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    let licenses = state
        .licenses
        .list_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let lic = licenses
        .into_iter()
        .find(|l| l.product_id == product.id)
        .ok_or(AppError::NotFound)?;

    Ok(Json(KeyResponse {
        license_key_prefix: lic.license_key_prefix,
    }))
}

#[derive(Debug, Serialize)]
pub struct DownloadCatalogRow {
    pub id: String,
    pub platform: String,
    pub version: String,
    pub sha256: String,
    pub size_bytes: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub released_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct DownloadsResponse {
    pub downloads: Vec<DownloadCatalogRow>,
}

pub async fn downloads_for_slug(
    State(state): State<AppState>,
    session: AuthSession,
    Path(slug): Path<String>,
) -> Result<Json<DownloadsResponse>, AppError> {
    let product = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    // Entitlement: user must hold an active license for this product.
    let licenses = state
        .licenses
        .list_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !licenses.iter().any(|l| l.product_id == product.id) {
        return Err(AppError::Forbidden);
    }

    let catalog = state
        .downloads_catalog
        .list_for_product(product.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let rows = catalog
        .into_iter()
        .map(|c| DownloadCatalogRow {
            id: c.id.to_string(),
            platform: c.platform,
            version: c.version,
            sha256: c.sha256,
            size_bytes: c.size_bytes,
            released_at: c.released_at,
        })
        .collect();
    Ok(Json(DownloadsResponse { downloads: rows }))
}
