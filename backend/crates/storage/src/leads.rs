//! `LeadsRepo` — `POST /v1/public/leads`.
//!
//! BACKEND.md §4 0018. Leads are append-only audit; we DO NOT dedupe on
//! `(email, source)`. The reasoning: a returning user re-submitting the
//! free-guide form is a valid funnel signal worth counting. Distinct counts
//! happen at query time in `/v1/admin/leads`.

use common::ids::LeadId;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use std::net::IpAddr;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Lead {
    pub id: LeadId,
    pub email: String,
    pub source: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct LeadsRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum LeadsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl LeadsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Admin list. Newest first. Limit caps at the call site; the
    /// 200-row default in /v1/admin/leads is enough for the in-app
    /// table; CSV export streams from the DB without this method.
    pub async fn list(&self, limit: i64) -> Result<Vec<Lead>, LeadsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, email::text AS "email!", source, created_at
            FROM leads
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Lead {
                id: LeadId::from_uuid(r.id),
                email: r.email,
                source: r.source,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn count_since(&self, since: OffsetDateTime) -> Result<i64, LeadsError> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM leads WHERE created_at >= $1"#,
            since,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count)
    }

    pub async fn create(
        &self,
        email: &str,
        source: &str,
        ip: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<Lead, LeadsError> {
        let id = uuid::Uuid::now_v7();
        let ip_net: Option<IpNetwork> = ip.map(Into::into);
        let row = sqlx::query!(
            r#"
            INSERT INTO leads (id, email, source, ip, user_agent)
            VALUES ($1, $2::citext, $3, $4, $5)
            RETURNING id, email::text AS "email!", source, created_at
            "#,
            id,
            email,
            source,
            ip_net,
            user_agent,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(Lead {
            id: LeadId::from_uuid(row.id),
            email: row.email,
            source: row.source,
            created_at: row.created_at,
        })
    }
}
