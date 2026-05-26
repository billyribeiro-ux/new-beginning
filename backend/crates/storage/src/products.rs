//! `ProductsRepo` — catalog reads for the public surface.
//!
//! BACKEND.md §4 0005. PR #6 ships the public reads; admin CRUD lands in
//! PR #14. JSONB / array columns ride through verbatim as `serde_json::Value`
//! / `Vec<String>` — the BFF maps to its UI shape.

use bigdecimal::BigDecimal;
use common::ids::ProductId;
use serde_json::Value as Json;
use sqlx::PgPool;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Product {
    pub id: ProductId,
    pub slug: String,
    pub legacy_slug_id: Option<String>,
    pub kind: String, // 'indicator' | 'course' — CHECK constraint
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub price_cents: i64,
    pub original_price_cents: Option<i64>,
    pub active: bool,
    pub badge: Option<String>,
    pub rating_value: BigDecimal,
    pub rating_count: i32,
    pub highlights: Vec<String>,
    pub specs_json: Json,
    pub deliverables: Vec<String>,
    pub requirements: Vec<String>,
    pub media_poster_color: String,
    pub media_accent: String,
    pub stripe_price_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct ProductsRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProductsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ProductsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List active products, optionally filtered by `kind`. Hits the
    /// `products_active_kind_idx` partial index when `kind` is `Some`.
    /// Admin list — includes inactive rows. Filterable by kind.
    pub async fn list_all(&self, kind: Option<&str>) -> Result<Vec<Product>, ProductsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, kind, name, tagline, description,
                   price_cents, original_price_cents, active, badge,
                   rating_value, rating_count, highlights,
                   specs_json AS "specs_json: Json",
                   deliverables, requirements, media_poster_color, media_accent,
                   stripe_price_id,
                   created_at, updated_at
            FROM products
            WHERE $1::text IS NULL OR kind = $1
            ORDER BY created_at DESC
            "#,
            kind
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Product {
                id: ProductId::from_uuid(r.id),
                slug: r.slug,
                legacy_slug_id: r.legacy_slug_id,
                kind: r.kind,
                name: r.name,
                tagline: r.tagline,
                description: r.description,
                price_cents: r.price_cents,
                original_price_cents: r.original_price_cents,
                active: r.active,
                badge: r.badge,
                rating_value: r.rating_value,
                rating_count: r.rating_count,
                highlights: r.highlights,
                specs_json: r.specs_json,
                deliverables: r.deliverables,
                requirements: r.requirements,
                media_poster_color: r.media_poster_color,
                media_accent: r.media_accent,
                stripe_price_id: r.stripe_price_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn list_active(&self, kind: Option<&str>) -> Result<Vec<Product>, ProductsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, kind, name, tagline, description,
                   price_cents, original_price_cents, active, badge,
                   rating_value, rating_count, highlights,
                   specs_json AS "specs_json: Json",
                   deliverables, requirements, media_poster_color, media_accent,
                   stripe_price_id,
                   created_at, updated_at
            FROM products
            WHERE active = TRUE AND ($1::text IS NULL OR kind = $1)
            ORDER BY name
            "#,
            kind
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Product {
                id: ProductId::from_uuid(r.id),
                slug: r.slug,
                legacy_slug_id: r.legacy_slug_id,
                kind: r.kind,
                name: r.name,
                tagline: r.tagline,
                description: r.description,
                price_cents: r.price_cents,
                original_price_cents: r.original_price_cents,
                active: r.active,
                badge: r.badge,
                rating_value: r.rating_value,
                rating_count: r.rating_count,
                highlights: r.highlights,
                specs_json: r.specs_json,
                deliverables: r.deliverables,
                requirements: r.requirements,
                media_poster_color: r.media_poster_color,
                media_accent: r.media_accent,
                stripe_price_id: r.stripe_price_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    /// Lookup by id; intentionally returns rows regardless of `active`
    /// — used by indicators/courses to resolve product metadata for a
    /// license/enrollment that may outlive the product going inactive.
    pub async fn find_by_id(&self, id: ProductId) -> Result<Option<Product>, ProductsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, kind, name, tagline, description,
                   price_cents, original_price_cents, active, badge,
                   rating_value, rating_count, highlights,
                   specs_json AS "specs_json: Json",
                   deliverables, requirements, media_poster_color, media_accent,
                   stripe_price_id,
                   created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Product {
            id: ProductId::from_uuid(r.id),
            slug: r.slug,
            legacy_slug_id: r.legacy_slug_id,
            kind: r.kind,
            name: r.name,
            tagline: r.tagline,
            description: r.description,
            price_cents: r.price_cents,
            original_price_cents: r.original_price_cents,
            active: r.active,
            badge: r.badge,
            rating_value: r.rating_value,
            rating_count: r.rating_count,
            highlights: r.highlights,
            specs_json: r.specs_json,
            deliverables: r.deliverables,
            requirements: r.requirements,
            media_poster_color: r.media_poster_color,
            media_accent: r.media_accent,
            stripe_price_id: r.stripe_price_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Toggle the `active` flag — admin PATCH primitive. Inactive
    /// products disappear from `/v1/public/products` but remain
    /// resolvable by id so licenses keep working.
    pub async fn set_active(&self, id: ProductId, active: bool) -> Result<u64, ProductsError> {
        let res = sqlx::query!(
            "UPDATE products SET active = $2, updated_at = now() WHERE id = $1",
            id.as_uuid(),
            active,
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Product>, ProductsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, kind, name, tagline, description,
                   price_cents, original_price_cents, active, badge,
                   rating_value, rating_count, highlights,
                   specs_json AS "specs_json: Json",
                   deliverables, requirements, media_poster_color, media_accent,
                   stripe_price_id,
                   created_at, updated_at
            FROM products
            WHERE slug = $1 AND active = TRUE
            "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Product {
            id: ProductId::from_uuid(r.id),
            slug: r.slug,
            legacy_slug_id: r.legacy_slug_id,
            kind: r.kind,
            name: r.name,
            tagline: r.tagline,
            description: r.description,
            price_cents: r.price_cents,
            original_price_cents: r.original_price_cents,
            active: r.active,
            badge: r.badge,
            rating_value: r.rating_value,
            rating_count: r.rating_count,
            highlights: r.highlights,
            specs_json: r.specs_json,
            deliverables: r.deliverables,
            requirements: r.requirements,
            media_poster_color: r.media_poster_color,
            media_accent: r.media_accent,
            stripe_price_id: r.stripe_price_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }
}
