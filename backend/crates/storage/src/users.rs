//! `UsersRepo` — concrete sqlx wrapper.
//!
//! BACKEND.md §5 (Plan agent's repo pattern): concrete struct, no trait, no
//! `Box<dyn>`. Tests get real Postgres via testcontainers (PR #4+); mocks are
//! not needed because we have exactly one backing store.
//!
//! PR #3 ships the slice auth needs: create + find_by_email + find_by_id.
//! Later PRs append (`update_password` in PR #4, `set_stripe_customer_id` in
//! PR #7, etc.) — never speculatively.

use common::ids::UserId;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub headline: Option<String>,
    pub timezone: String,
    pub language: String,
    pub role: String, // "member" | "admin" — CHECK constraint in 0002_users.sql.
    pub password_hash: Option<String>,
    pub email_verified_at: Option<OffsetDateTime>,
    pub totp_enabled_at: Option<OffsetDateTime>,
    pub stripe_customer_id: Option<String>,
    pub created_at: OffsetDateTime,
}

/// Patch fields. `None` means "don't touch this column"; `Some(_)` means
/// "set it to this value" (including `Some(None)` to NULL `headline`).
#[derive(Debug, Default, Clone)]
pub struct UserPatch {
    pub name: Option<String>,
    pub headline: Option<Option<String>>,
    pub timezone: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone)]
pub struct UsersRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum UsersError {
    #[error("user with that email already exists")]
    DuplicateEmail,

    #[error("user not found")]
    NotFound,

    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl UsersRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, UsersError> {
        let row = sqlx::query!(
            r#"
            SELECT id, email::text AS "email!", name, headline, timezone, language,
                   role, password_hash,
                   email_verified_at, totp_enabled_at, stripe_customer_id, created_at
            FROM users
            WHERE email = $1::citext AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| User {
            id: UserId::from_uuid(r.id),
            email: r.email,
            name: r.name,
            headline: r.headline,
            timezone: r.timezone,
            language: r.language,
            role: r.role,
            password_hash: r.password_hash,
            email_verified_at: r.email_verified_at,
            totp_enabled_at: r.totp_enabled_at,
            stripe_customer_id: r.stripe_customer_id,
            created_at: r.created_at,
        }))
    }

    pub async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UsersError> {
        let row = sqlx::query!(
            r#"
            SELECT id, email::text AS "email!", name, headline, timezone, language,
                   role, password_hash,
                   email_verified_at, totp_enabled_at, stripe_customer_id, created_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| User {
            id: UserId::from_uuid(r.id),
            email: r.email,
            name: r.name,
            headline: r.headline,
            timezone: r.timezone,
            language: r.language,
            role: r.role,
            password_hash: r.password_hash,
            email_verified_at: r.email_verified_at,
            totp_enabled_at: r.totp_enabled_at,
            stripe_customer_id: r.stripe_customer_id,
            created_at: r.created_at,
        }))
    }

    /// Insert a new user. The email is case-insensitive (CITEXT) and the
    /// UNIQUE constraint surfaces collision as `UsersError::DuplicateEmail`,
    /// distinct from other DB errors.
    /// Count signups since `cutoff`. Admin KPI; used by
    /// `/v1/admin/stats`.
    pub async fn count_since(&self, cutoff: time::OffsetDateTime) -> Result<i64, UsersError> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!" FROM users WHERE created_at >= $1"#,
            cutoff,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count)
    }

    /// Admin customer search by email substring (case-insensitive via
    /// CITEXT comparison). Limit caps; UI typically searches with a few
    /// chars and an autocomplete.
    pub async fn search_by_email(&self, query: &str, limit: i64) -> Result<Vec<User>, UsersError> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query!(
            r#"
            SELECT id, email::text AS "email!", name, headline, timezone, language,
                   role, password_hash,
                   email_verified_at, totp_enabled_at, stripe_customer_id, created_at
            FROM users
            WHERE email ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            pattern,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| User {
                id: UserId::from_uuid(r.id),
                email: r.email,
                name: r.name,
                headline: r.headline,
                timezone: r.timezone,
                language: r.language,
                role: r.role,
                password_hash: r.password_hash,
                email_verified_at: r.email_verified_at,
                totp_enabled_at: r.totp_enabled_at,
                stripe_customer_id: r.stripe_customer_id,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Soft-delete: stamp `deleted_at` + revoke every active session.
    /// Other rows (orders/invoices/audit) are retained — legal /
    /// accounting reasons. CASCADE on FKs in 0002 means actual hard
    /// purge happens in a follow-up retention job.
    pub async fn soft_delete(&self, id: UserId) -> Result<u64, UsersError> {
        let mut tx = self.pool.begin().await?;
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = now(),
                email = 'deleted-' || id::text || '@deleted.tradeflex.invalid',
                password_hash = NULL,
                totp_enabled_at = NULL,
                stripe_customer_id = NULL
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid(),
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", id.as_uuid(),)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(res.rows_affected())
    }

    pub async fn create(
        &self,
        email: &str,
        name: &str,
        password_hash: &str,
    ) -> Result<User, UsersError> {
        let id: Uuid = Uuid::now_v7();
        let row = sqlx::query!(
            r#"
            INSERT INTO users (id, email, name, password_hash)
            VALUES ($1, $2::citext, $3, $4)
            RETURNING id, email::text AS "email!", name, headline, timezone, language,
                      role, password_hash,
                      email_verified_at, totp_enabled_at, stripe_customer_id, created_at
            "#,
            id,
            email,
            name,
            password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db) if db.constraint() == Some("users_email_key") => {
                UsersError::DuplicateEmail
            }
            _ => UsersError::Sqlx(e),
        })?;
        Ok(User {
            id: UserId::from_uuid(row.id),
            email: row.email,
            name: row.name,
            headline: row.headline,
            timezone: row.timezone,
            language: row.language,
            role: row.role,
            password_hash: row.password_hash,
            email_verified_at: row.email_verified_at,
            totp_enabled_at: row.totp_enabled_at,
            stripe_customer_id: row.stripe_customer_id,
            created_at: row.created_at,
        })
    }

    /// Apply a sparse profile patch. Uses `COALESCE($i, column)` to leave
    /// untouched fields alone while still using a single UPDATE.
    /// `headline` is a tri-state (Some(Some) = set, Some(None) = NULL,
    /// None = leave) — handled via a separate boolean param.
    pub async fn update_profile(&self, id: UserId, patch: UserPatch) -> Result<User, UsersError> {
        let (clear_headline, new_headline) = match patch.headline {
            Some(Some(v)) => (false, Some(v)),
            Some(None) => (true, None),
            None => (false, None),
        };
        let row = sqlx::query!(
            r#"
            UPDATE users SET
                name      = COALESCE($2, name),
                headline  = CASE
                              WHEN $3::boolean THEN NULL
                              WHEN $4::text IS NOT NULL THEN $4
                              ELSE headline
                            END,
                timezone  = COALESCE($5, timezone),
                language  = COALESCE($6, language),
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, email::text AS "email!", name, headline, timezone, language,
                      role, password_hash,
                      email_verified_at, totp_enabled_at, stripe_customer_id, created_at
            "#,
            id.as_uuid(),
            patch.name,
            clear_headline,
            new_headline,
            patch.timezone,
            patch.language,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UsersError::NotFound)?;
        Ok(User {
            id: UserId::from_uuid(row.id),
            email: row.email,
            name: row.name,
            headline: row.headline,
            timezone: row.timezone,
            language: row.language,
            role: row.role,
            password_hash: row.password_hash,
            email_verified_at: row.email_verified_at,
            totp_enabled_at: row.totp_enabled_at,
            stripe_customer_id: row.stripe_customer_id,
            created_at: row.created_at,
        })
    }

    /// Attach a Stripe `cus_…` id to the user. UNIQUE constraint on the
    /// column means a second call with the same id is a no-op; a different
    /// id surfaces as a duplicate-key error (which is a bug).
    pub async fn set_stripe_customer_id(
        &self,
        id: UserId,
        customer_id: &str,
    ) -> Result<(), UsersError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET stripe_customer_id = COALESCE(stripe_customer_id, $2),
                updated_at = now()
            WHERE id = $1
            "#,
            id.as_uuid(),
            customer_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Replace a user's `password_hash`. Callers MUST verify the current
    /// password (or otherwise prove ownership) before calling.
    pub async fn update_password(&self, id: UserId, new_hash: &str) -> Result<(), UsersError> {
        let res = sqlx::query!(
            r#"
            UPDATE users SET password_hash = $2, updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid(),
            new_hash,
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(UsersError::NotFound);
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // 2FA (TOTP + backup codes)  — PR #5
    // ------------------------------------------------------------------

    /// Stage an encrypted TOTP secret. Resets `totp_enabled_at` to NULL —
    /// the user must re-confirm with `enable_totp` before 2FA is active.
    /// Used by `POST /v1/me/2fa/enable`.
    pub async fn set_totp_secret(
        &self,
        id: UserId,
        encrypted_secret: &[u8],
    ) -> Result<(), UsersError> {
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET totp_secret_encrypted = $2,
                totp_enabled_at = NULL,
                twofa_backup_codes_hash = NULL,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid(),
            encrypted_secret,
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(UsersError::NotFound);
        }
        Ok(())
    }

    /// Mark TOTP as confirmed and persist the backup-code hashes. Atomic —
    /// either both columns flip or neither does. Used by
    /// `POST /v1/me/2fa/confirm`.
    pub async fn enable_totp(
        &self,
        id: UserId,
        backup_codes: &[Vec<u8>],
    ) -> Result<(), UsersError> {
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET totp_enabled_at = now(),
                twofa_backup_codes_hash = $2,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL AND totp_secret_encrypted IS NOT NULL
            "#,
            id.as_uuid(),
            backup_codes,
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(UsersError::NotFound);
        }
        Ok(())
    }

    /// Clear all 2FA state. Used by `POST /v1/me/2fa/disable`.
    pub async fn disable_totp(&self, id: UserId) -> Result<(), UsersError> {
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET totp_secret_encrypted = NULL,
                totp_enabled_at = NULL,
                twofa_backup_codes_hash = NULL,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid()
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(UsersError::NotFound);
        }
        Ok(())
    }

    /// Replace all backup-code hashes. Used by
    /// `POST /v1/me/2fa/backup-codes/regenerate`.
    pub async fn replace_backup_codes(
        &self,
        id: UserId,
        backup_codes: &[Vec<u8>],
    ) -> Result<(), UsersError> {
        let res = sqlx::query!(
            r#"
            UPDATE users SET twofa_backup_codes_hash = $2, updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL AND totp_enabled_at IS NOT NULL
            "#,
            id.as_uuid(),
            backup_codes,
        )
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(UsersError::NotFound);
        }
        Ok(())
    }

    /// Fetch the encrypted TOTP secret bytes (`Some` iff `set_totp_secret`
    /// or `enable_totp` has been called and 2FA isn't disabled). Decryption
    /// happens at the call site so the key never crosses the storage
    /// boundary.
    pub async fn get_totp_secret(&self, id: UserId) -> Result<Option<Vec<u8>>, UsersError> {
        let row = sqlx::query!(
            r#"
            SELECT totp_secret_encrypted FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(|r| r.totp_secret_encrypted))
    }

    /// Fetch every backup-code slot (consumed slots included, so the caller
    /// can `is_zeroed_slot` to render "X/10 remaining"). Returns an empty
    /// vec when 2FA is off.
    pub async fn get_backup_codes(&self, id: UserId) -> Result<Vec<Vec<u8>>, UsersError> {
        let row = sqlx::query!(
            r#"
            SELECT twofa_backup_codes_hash FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .and_then(|r| r.twofa_backup_codes_hash)
            .unwrap_or_default())
    }

    /// Atomically replace `twofa_backup_codes_hash[index]` with a sentinel
    /// "consumed" value (all-zero bytes). The CAS is on the pre-image bytes,
    /// so two concurrent uses of the same backup code race: at most one
    /// wins, and the other observes the consumed slot on retry.
    ///
    /// Returns `Ok(true)` on consume; `Ok(false)` if the slot moved (CAS
    /// failed — somebody else used the code first).
    pub async fn consume_backup_code_slot(
        &self,
        id: UserId,
        index: i32,
        expected_hash: &[u8],
        new_zeroed: &[u8],
    ) -> Result<bool, UsersError> {
        // Postgres array is 1-indexed in SQL; the caller passes 0-indexed
        // and we add 1 here so the contract matches Rust's convention.
        let sql_index = index + 1;
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET twofa_backup_codes_hash[$2] = $3,
                updated_at = now()
            WHERE id = $1
              AND deleted_at IS NULL
              AND totp_enabled_at IS NOT NULL
              AND twofa_backup_codes_hash[$2] = $4
            "#,
            id.as_uuid(),
            sql_index,
            new_zeroed,
            expected_hash,
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() == 1)
    }
}
