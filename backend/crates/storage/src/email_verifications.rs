//! `EmailVerificationsRepo` — stages email-change / signup-verify / password-
//! reset tokens.
//!
//! BACKEND.md §4 0004: `kind IN ('signup','email_change','password_reset')`;
//! `new_email` populated iff `kind = 'email_change'`. Tokens are stored as
//! `sha256(token)` only; the plaintext goes in the link the mailer sends.

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use common::ids::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationKind {
    Signup,
    EmailChange,
    PasswordReset,
}

impl VerificationKind {
    fn as_str(self) -> &'static str {
        match self {
            VerificationKind::Signup => "signup",
            VerificationKind::EmailChange => "email_change",
            VerificationKind::PasswordReset => "password_reset",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingVerification {
    pub id: Uuid,
    pub user_id: UserId,
    pub kind: String,
    pub new_email: Option<String>,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct EmailVerificationsRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailVerificationsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl EmailVerificationsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new verification row. Returns the row's id + expires_at; the
    /// caller already holds the plaintext token (we only stored its sha256).
    pub async fn create(
        &self,
        user_id: UserId,
        kind: VerificationKind,
        token_hash: &[u8; 32],
        new_email: Option<&str>,
        ttl: Duration,
    ) -> Result<PendingVerification, EmailVerificationsError> {
        let id = Uuid::now_v7();
        let expires_at = OffsetDateTime::now_utc() + ttl;
        let row = sqlx::query!(
            r#"
            INSERT INTO email_verifications
                (id, user_id, token_hash, kind, new_email, expires_at)
            VALUES
                ($1, $2, $3, $4, $5::citext, $6)
            RETURNING id, user_id, kind, new_email::text AS new_email, expires_at
            "#,
            id,
            user_id.as_uuid(),
            &token_hash[..],
            kind.as_str(),
            new_email,
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(PendingVerification {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            kind: row.kind,
            new_email: row.new_email,
            expires_at: row.expires_at,
        })
    }
}
