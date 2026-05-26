//! `GET /v1/public/products[?kind=indicator|course]` and
//! `GET /v1/public/products/{slug}`.
//!
//! BACKEND.md §12: `PublicRead` bucket — 60/min/IP.

use axum::extract::{ConnectInfo, Path, Query, State};
use axum::Json;
use common::error::AppError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;

use crate::middleware::rate_limit::Bucket;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// `indicator` | `course`. Other values rejected (400).
    pub kind: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub products: Vec<ProductPayload>,
}

#[derive(Debug, Serialize)]
pub struct ProductPayload {
    pub id: String,
    pub slug: String,
    pub kind: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub price_cents: i64,
    pub original_price_cents: Option<i64>,
    pub badge: Option<String>,
    pub rating: RatingPayload,
    pub highlights: Vec<String>,
    pub specs: serde_json::Value,
    pub deliverables: Vec<String>,
    pub requirements: Vec<String>,
    pub media: MediaPayload,
}

#[derive(Debug, Serialize)]
pub struct RatingPayload {
    pub value: f64,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct MediaPayload {
    pub poster_color: String,
    pub accent: String,
}

impl ProductPayload {
    fn from_repo(p: storage::Product) -> Self {
        // BigDecimal → f64 for JSON. The CHECK constraint on
        // `rating_value` keeps it in [0.0, 5.0] so the lossy cast cannot
        // produce values the BFF doesn't expect.
        let rating_value = p.rating_value.to_string().parse::<f64>().unwrap_or(0.0);
        Self {
            id: p.id.to_string(),
            slug: p.slug,
            kind: p.kind,
            name: p.name,
            tagline: p.tagline,
            description: p.description,
            price_cents: p.price_cents,
            original_price_cents: p.original_price_cents,
            badge: p.badge,
            rating: RatingPayload {
                value: rating_value,
                count: p.rating_count,
            },
            highlights: p.highlights,
            specs: p.specs_json,
            deliverables: p.deliverables,
            requirements: p.requirements,
            media: MediaPayload {
                poster_color: p.media_poster_color,
                accent: p.media_accent,
            },
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    state
        .limiter
        .check(Bucket::PublicRead, peer.ip())
        .map_err(|e| AppError::RateLimited {
            retry_after_secs: e.retry_after_secs,
        })?;

    let kind_filter = match q.kind.as_deref() {
        None => None,
        Some("indicator") | Some("course") => q.kind.as_deref(),
        Some(other) => {
            return Err(AppError::Validation(format!(
                "kind must be 'indicator' or 'course', got '{other}'"
            )));
        }
    };

    let rows = state
        .products
        .list_active(kind_filter)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        products: rows.into_iter().map(ProductPayload::from_repo).collect(),
    }))
}

pub async fn detail(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Path(slug): Path<String>,
) -> Result<Json<ProductPayload>, AppError> {
    state
        .limiter
        .check(Bucket::PublicRead, peer.ip())
        .map_err(|e| AppError::RateLimited {
            retry_after_secs: e.retry_after_secs,
        })?;

    // Slug syntax guard: prevent silly inputs from hitting the DB.
    if slug.is_empty()
        || slug.len() > 128
        || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(AppError::NotFound);
    }
    let _ = String::from_str(&slug).ok();

    let row = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    match row {
        Some(p) => Ok(Json(ProductPayload::from_repo(p))),
        None => Err(AppError::NotFound),
    }
}
