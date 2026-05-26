//! TradeFlex catalog seeder.
//!
//! BACKEND.md §1.6 + §21 row 2. Reads `backend/seeds/catalog.json` (produced
//! by `pnpm run dump-catalog`) and upserts into `products` +
//! `subscription_plans`. Idempotent — `ON CONFLICT (slug) DO UPDATE` lets it
//! be re-run any number of times.
//!
//! Catalog rows are admin-controlled facts, not user data, so generating UUIDs
//! per slug deterministically (so a re-run produces the same `id`) is a goal:
//! we derive the UUID from the slug via SHA-256 then format as v8 (custom).
//! However the simpler approach is: if a row with that slug already exists,
//! keep its id; otherwise mint a fresh `now_v7()`. We take the simpler path.

use anyhow::Context;
use bigdecimal::BigDecimal;
use common::config::Config;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

const SERVICE_NAME: &str = "tradeflex-seeder";
const CATALOG_PATH_ENV: &str = "CATALOG_PATH";
const DEFAULT_CATALOG_PATH: &str = "seeds/catalog.json";

#[derive(Debug, Deserialize)]
struct Catalog {
    indicators: Vec<ProductInput>,
    courses: Vec<ProductInput>,
    plans: Vec<PlanInput>,
}

#[derive(Debug, Deserialize)]
struct ProductInput {
    /// Carry the existing `prod_revolution_ranger`-style ID for traceability.
    legacy_slug_id: String,
    slug: String,
    kind: String, // 'indicator' | 'course'
    name: String,
    tagline: String,
    description: String,
    price_cents: i64,
    #[serde(default)]
    original_price_cents: Option<i64>,
    #[serde(default = "default_active")]
    active: bool,
    #[serde(default)]
    badge: Option<String>,
    #[serde(default)]
    rating_value: Option<f64>,
    #[serde(default)]
    rating_count: Option<i32>,
    #[serde(default)]
    highlights: Vec<String>,
    #[serde(default)]
    specs_json: serde_json::Value,
    #[serde(default)]
    deliverables: Vec<String>,
    #[serde(default)]
    requirements: Vec<String>,
    media_poster_color: String,
    media_accent: String,
}

#[derive(Debug, Deserialize)]
struct PlanInput {
    legacy_slug_id: String,
    slug: String,
    name: String,
    cadence: String, // 'monthly' | 'quarterly' | 'annual'
    price_cents: i64,
    monthly_equivalent_cents: i64,
    #[serde(default)]
    savings_pct: i32,
    tagline: String,
    #[serde(default)]
    highlights: Vec<String>,
    #[serde(default)]
    featured: bool,
    #[serde(default)]
    badge: Option<String>,
    /// Stripe price id is required by the schema. Until PR #7 attaches real
    /// Stripe prices, the seeder emits a deterministic placeholder so the
    /// schema's `NOT NULL UNIQUE` constraint is satisfied.
    #[serde(default)]
    stripe_price_id: Option<String>,
}

fn default_active() -> bool {
    true
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().context("load config")?;
    let _obs = observability::init(&config, SERVICE_NAME)?;

    let pool = storage::build_pool(&config).await.context("connect db")?;
    tracing::info!("connected, applying migrations before seed");
    storage::migrate::run(&pool).await.context("migrate")?;

    let catalog_path: PathBuf = std::env::var(CATALOG_PATH_ENV)
        .unwrap_or_else(|_| DEFAULT_CATALOG_PATH.to_string())
        .into();
    let catalog_bytes =
        std::fs::read(&catalog_path).with_context(|| format!("read {}", catalog_path.display()))?;
    let catalog: Catalog = serde_json::from_slice(&catalog_bytes).context("parse catalog json")?;

    tracing::info!(
        indicators = catalog.indicators.len(),
        courses = catalog.courses.len(),
        plans = catalog.plans.len(),
        path = %catalog_path.display(),
        "seeding catalog",
    );

    let mut tx = pool.begin().await?;

    for p in catalog.indicators.iter().chain(catalog.courses.iter()) {
        upsert_product(&mut tx, p).await?;
    }
    for plan in &catalog.plans {
        upsert_plan(&mut tx, plan).await?;
    }

    tx.commit().await?;
    tracing::info!("seed complete");

    let counts = sqlx::query!(
        r#"SELECT
            (SELECT count(*) FROM products) AS "products!",
            (SELECT count(*) FROM subscription_plans) AS "plans!""#
    )
    .fetch_one(&pool)
    .await?;
    tracing::info!(
        products = counts.products,
        plans = counts.plans,
        "row counts after seed"
    );
    Ok(())
}

async fn upsert_product(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    p: &ProductInput,
) -> anyhow::Result<()> {
    let id = Uuid::now_v7();
    // BigDecimal so the param matches the column's NUMERIC(3,2) type. Format
    // with two decimals to stay inside the schema scale; the column's CHECK
    // constraint enforces 0..=5.
    let rating_value = BigDecimal::from_str(&format!("{:.2}", p.rating_value.unwrap_or(0.0)))
        .context("rating_value to decimal")?;
    let rating_count = p.rating_count.unwrap_or(0);

    // PR #8 placeholder: the column is `TEXT UNIQUE` so we set a
    // deterministic per-slug fake value. PR #14's admin product CRUD or a
    // one-off sync script overwrites these with real Stripe price ids.
    let stripe_price_id = format!("price_test_TF_{}", p.slug);

    sqlx::query!(
        r#"
        INSERT INTO products (
            id, slug, legacy_slug_id, kind, name, tagline, description,
            price_cents, original_price_cents, active, badge,
            rating_value, rating_count, highlights, specs_json,
            deliverables, requirements, media_poster_color, media_accent,
            stripe_price_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11,
            $12, $13, $14, $15,
            $16, $17, $18, $19,
            $20
        )
        ON CONFLICT (slug) DO UPDATE SET
            legacy_slug_id       = EXCLUDED.legacy_slug_id,
            kind                 = EXCLUDED.kind,
            name                 = EXCLUDED.name,
            tagline              = EXCLUDED.tagline,
            description          = EXCLUDED.description,
            price_cents          = EXCLUDED.price_cents,
            original_price_cents = EXCLUDED.original_price_cents,
            active               = EXCLUDED.active,
            badge                = EXCLUDED.badge,
            rating_value         = EXCLUDED.rating_value,
            rating_count         = EXCLUDED.rating_count,
            highlights           = EXCLUDED.highlights,
            specs_json           = EXCLUDED.specs_json,
            deliverables         = EXCLUDED.deliverables,
            requirements         = EXCLUDED.requirements,
            media_poster_color   = EXCLUDED.media_poster_color,
            media_accent         = EXCLUDED.media_accent,
            stripe_price_id      = COALESCE(products.stripe_price_id, EXCLUDED.stripe_price_id),
            updated_at           = now()
        "#,
        id,
        p.slug,
        p.legacy_slug_id,
        p.kind,
        p.name,
        p.tagline,
        p.description,
        p.price_cents,
        p.original_price_cents,
        p.active,
        p.badge,
        rating_value,
        rating_count,
        &p.highlights,
        p.specs_json,
        &p.deliverables,
        &p.requirements,
        p.media_poster_color,
        p.media_accent,
        stripe_price_id,
    )
    .execute(&mut **tx)
    .await?;
    tracing::debug!(slug = %p.slug, "upserted product");
    Ok(())
}

async fn upsert_plan(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    p: &PlanInput,
) -> anyhow::Result<()> {
    let id = Uuid::now_v7();
    let stripe_price_id = p
        .stripe_price_id
        .clone()
        .unwrap_or_else(|| format!("price_placeholder_{}", p.slug));

    sqlx::query!(
        r#"
        INSERT INTO subscription_plans (
            id, slug, legacy_slug_id, name, cadence,
            price_cents, monthly_equivalent_cents, savings_pct,
            tagline, highlights, featured, badge, stripe_price_id
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8,
            $9, $10, $11, $12, $13
        )
        ON CONFLICT (slug) DO UPDATE SET
            legacy_slug_id           = EXCLUDED.legacy_slug_id,
            name                     = EXCLUDED.name,
            cadence                  = EXCLUDED.cadence,
            price_cents              = EXCLUDED.price_cents,
            monthly_equivalent_cents = EXCLUDED.monthly_equivalent_cents,
            savings_pct              = EXCLUDED.savings_pct,
            tagline                  = EXCLUDED.tagline,
            highlights               = EXCLUDED.highlights,
            featured                 = EXCLUDED.featured,
            badge                    = EXCLUDED.badge,
            stripe_price_id          = EXCLUDED.stripe_price_id,
            updated_at               = now()
        "#,
        id,
        p.slug,
        p.legacy_slug_id,
        p.name,
        p.cadence,
        p.price_cents,
        p.monthly_equivalent_cents,
        p.savings_pct,
        p.tagline,
        &p.highlights,
        p.featured,
        p.badge,
        stripe_price_id,
    )
    .execute(&mut **tx)
    .await?;
    tracing::debug!(slug = %p.slug, "upserted plan");
    Ok(())
}
