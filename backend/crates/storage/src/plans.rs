//! `PlansRepo` — subscription-plan catalog reads.
//!
//! BACKEND.md §4 0006.

use common::ids::PlanId;
use sqlx::PgPool;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct SubscriptionPlan {
    pub id: PlanId,
    pub slug: String,
    pub legacy_slug_id: Option<String>,
    pub name: String,
    pub cadence: String, // 'monthly' | 'quarterly' | 'annual'
    pub price_cents: i64,
    pub monthly_equivalent_cents: i64,
    pub savings_pct: i32,
    pub tagline: String,
    pub highlights: Vec<String>,
    pub featured: bool,
    pub badge: Option<String>,
    pub stripe_price_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct PlansRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum PlansError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl PlansRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List every plan. The pricing page renders all three cadences so a
    /// LIMIT here would be wrong — return them sorted by cadence so the UI
    /// has a stable order (monthly < quarterly < annual).
    pub async fn list_all(&self) -> Result<Vec<SubscriptionPlan>, PlansError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, name, cadence, price_cents,
                   monthly_equivalent_cents, savings_pct, tagline, highlights,
                   featured, badge, stripe_price_id, created_at, updated_at
            FROM subscription_plans
            ORDER BY price_cents
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| SubscriptionPlan {
                id: PlanId::from_uuid(r.id),
                slug: r.slug,
                legacy_slug_id: r.legacy_slug_id,
                name: r.name,
                cadence: r.cadence,
                price_cents: r.price_cents,
                monthly_equivalent_cents: r.monthly_equivalent_cents,
                savings_pct: r.savings_pct,
                tagline: r.tagline,
                highlights: r.highlights,
                featured: r.featured,
                badge: r.badge,
                stripe_price_id: r.stripe_price_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<SubscriptionPlan>, PlansError> {
        let row = sqlx::query!(
            r#"
            SELECT id, slug, legacy_slug_id, name, cadence, price_cents,
                   monthly_equivalent_cents, savings_pct, tagline, highlights,
                   featured, badge, stripe_price_id, created_at, updated_at
            FROM subscription_plans
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| SubscriptionPlan {
            id: PlanId::from_uuid(r.id),
            slug: r.slug,
            legacy_slug_id: r.legacy_slug_id,
            name: r.name,
            cadence: r.cadence,
            price_cents: r.price_cents,
            monthly_equivalent_cents: r.monthly_equivalent_cents,
            savings_pct: r.savings_pct,
            tagline: r.tagline,
            highlights: r.highlights,
            featured: r.featured,
            badge: r.badge,
            stripe_price_id: r.stripe_price_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }
}
