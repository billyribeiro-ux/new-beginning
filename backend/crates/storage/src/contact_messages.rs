//! `ContactRepo` — `POST /v1/public/contact`.
//!
//! BACKEND.md §4 0018.

use common::ids::ContactMessageId;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use std::net::IpAddr;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ContactMessage {
    pub id: ContactMessageId,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub status: String, // 'new' | 'read' | 'archived' | 'spam'
    pub created_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct ContactRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum ContactError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl ContactRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        status_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ContactMessage>, ContactError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, email::text AS "email!", subject, body, status, created_at
            FROM contact_messages
            WHERE $1::text IS NULL OR status = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            status_filter,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| ContactMessage {
                id: ContactMessageId::from_uuid(r.id),
                name: r.name,
                email: r.email,
                subject: r.subject,
                body: r.body,
                status: r.status,
                created_at: r.created_at,
            })
            .collect())
    }

    /// PATCH /v1/admin/messages/{id} — flip status. Enforces the closed
    /// set via the schema CHECK. Returns rows_affected for the handler
    /// to map to 200/404.
    pub async fn update_status(
        &self,
        id: ContactMessageId,
        status: &str,
    ) -> Result<u64, ContactError> {
        let res = sqlx::query!(
            r#"UPDATE contact_messages SET status = $2 WHERE id = $1"#,
            id.as_uuid(),
            status,
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn create(
        &self,
        name: &str,
        email: &str,
        subject: &str,
        body: &str,
        ip: Option<IpAddr>,
    ) -> Result<ContactMessage, ContactError> {
        let id = uuid::Uuid::now_v7();
        let ip_net: Option<IpNetwork> = ip.map(Into::into);
        let row = sqlx::query!(
            r#"
            INSERT INTO contact_messages (id, name, email, subject, body, ip)
            VALUES ($1, $2, $3::citext, $4, $5, $6)
            RETURNING id, name, email::text AS "email!", subject, body, status, created_at
            "#,
            id,
            name,
            email,
            subject,
            body,
            ip_net,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ContactMessage {
            id: ContactMessageId::from_uuid(row.id),
            name: row.name,
            email: row.email,
            subject: row.subject,
            body: row.body,
            status: row.status,
            created_at: row.created_at,
        })
    }
}
