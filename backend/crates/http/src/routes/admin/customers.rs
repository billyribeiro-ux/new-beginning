//! `GET /v1/admin/customers?q=` and
//! `POST /v1/admin/customers/{id}/grant-entitlement` — admin search +
//! manual entitlement grant.
//!
//! BACKEND.md §11.

use axum::extract::{Path, Query, State};
use axum::Json;
use common::error::AppError;
use common::ids::{ProductId, UserId};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CustomerRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub customers: Vec<CustomerRow>,
}

const MAX_LIMIT: i64 = 200;
const DEFAULT_LIMIT: i64 = 50;

pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let query = q.q.unwrap_or_default();
    if query.trim().is_empty() {
        return Ok(Json(SearchResponse { customers: vec![] }));
    }
    let users = state
        .users
        .search_by_email(query.trim(), limit)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(SearchResponse {
        customers: users
            .into_iter()
            .map(|u| CustomerRow {
                id: u.id.to_string(),
                email: u.email,
                name: u.name,
                role: u.role,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct GrantRequest {
    pub product_id: ProductId,
    /// Free-form reason. Recorded in the audit row so the manual grant
    /// can be traced back to a support ticket / refund / cohort
    /// migration. Required so a grant never lands without a "why".
    #[validate(length(min = 4, max = 500))]
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct GrantResponse {
    /// The freshly-issued license key prefix. The plaintext lives only
    /// in this response — surface it to the admin once, then it's gone
    /// (matches the user-issuance flow).
    pub license_key_prefix: String,
    pub license_id: String,
    pub enrollment_id_if_created: Option<String>,
}

pub async fn grant_entitlement(
    State(state): State<AppState>,
    session: AuthSession,
    Path(user_id): Path<UserId>,
    Json(req): Json<GrantRequest>,
) -> Result<Json<GrantResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Resolve target user (also acts as a 404 guard).
    let user = state
        .users
        .find_by_id(user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    // Resolve target product (any kind — courses get an enrollment,
    // indicators get a license, both is fine).
    let product = state
        .products
        .find_by_id(req.product_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    // Manual grants synthesize a unique source_ref_id so they don't
    // collide with a real order_id (which is what
    // `issue_for_purchase_in_tx`'s skip-existing check keys off of).
    let synthetic_order_id = common::ids::OrderId::new();
    let kind_prefix = short_kind(&product.kind);

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // Issue a license (idempotent only on the synthetic ref id — a
    // repeat call would mint a new one. Acceptable for manual grants).
    let issued = state
        .licenses
        .issue_for_purchase_in_tx(
            &mut tx,
            &SecretString::from(license_pepper(&state)),
            user.id,
            product.id,
            synthetic_order_id,
            kind_prefix,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "license issuance unexpectedly returned None"
            ))
        })?;

    // Enrollment if the product is a course. (For indicators, the
    // license IS the entitlement; an enrollment row would be inert.)
    let enrollment_id_if_created = if product.kind == "course" {
        state
            .enrollments
            .create_for_purchase_in_tx(&mut tx, user.id, product.id, synthetic_order_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let enr = state
            .enrollments
            .find_for_user_and_product(user.id, product.id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        enr.map(|e| e.id.to_string())
    } else {
        None
    };

    state
        .audit
        .record_in_tx(
            &mut tx,
            Some(session.user_id),
            "admin.entitlement.granted",
            "users",
            &user.id.to_string(),
            serde_json::json!({
                "product_id": product.id.to_string(),
                "product_slug": product.slug,
                "kind": product.kind,
                "license_prefix": issued.prefix,
                "synthetic_order_id": synthetic_order_id.to_string(),
                "reason": req.reason,
            }),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(GrantResponse {
        license_key_prefix: issued.prefix,
        license_id: issued.id.to_string(),
        enrollment_id_if_created,
    }))
}

fn license_pepper(state: &AppState) -> String {
    use secrecy::ExposeSecret;
    state.password_pepper.expose_secret().to_string()
}

/// Mirror of dispatch's helper — turn a product kind slug into the 2-char
/// license-key prefix segment ("indicator" → "IN", "course" → "CR").
/// Duplicated here to avoid creating a public re-export from the
/// dispatcher; will be lifted to a shared module if a third call site
/// appears.
fn short_kind(kind: &str) -> &'static str {
    match kind {
        "indicator" => "IN",
        "course" => "CR",
        _ => "OT",
    }
}
