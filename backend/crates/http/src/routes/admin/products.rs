//! `GET /v1/admin/products` + `PATCH /v1/admin/products/{id}` (active flag).
//!
//! Full CRUD (create / replace / delete / duplicate) is a follow-up;
//! today the admin can only flip `active`. Editing other fields lands
//! once the admin UI ships an editor pane.

use axum::extract::{Path, Query, State};
use axum::Json;
use common::error::AppError;
use common::ids::ProductId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductRow {
    pub id: String,
    pub slug: String,
    pub kind: String,
    pub name: String,
    pub price_cents: i64,
    pub active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub products: Vec<ProductRow>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let products = state
        .products
        .list_all(q.kind.as_deref())
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        products: products
            .into_iter()
            .map(|p| ProductRow {
                id: p.id.to_string(),
                slug: p.slug,
                kind: p.kind,
                name: p.name,
                price_cents: p.price_cents,
                active: p.active,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct PatchRequest {
    pub active: Option<bool>,
}

pub async fn patch(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<ProductId>,
    Json(req): Json<PatchRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let Some(new_active) = req.active else {
        return Err(AppError::Validation("nothing to update".into()));
    };
    // Audit + flag flip in one tx (BACKEND.md §22 rule 8).
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let n = sqlx::query!(
        "UPDATE products SET active = $2, updated_at = now() WHERE id = $1",
        id.as_uuid(),
        new_active,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    .rows_affected();
    if n == 0 {
        let _ = tx.rollback().await;
        return Err(AppError::NotFound);
    }
    state
        .audit
        .record_in_tx(
            &mut tx,
            Some(session.user_id),
            if new_active {
                "admin.product.activated"
            } else {
                "admin.product.deactivated"
            },
            "products",
            &id.to_string(),
            serde_json::json!({ "active": new_active }),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
