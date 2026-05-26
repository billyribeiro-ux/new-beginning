//! `EnrollmentsRepo` + `LicensesRepo` — the entitlements every paid order
//! produces.
//!
//! BACKEND.md §8.5: a `purchase`-source enrollment / license carries the
//! parent `order_id` in `source_ref_id` so PR #15's refund handler can
//! revoke it deterministically.
//!
//! License keys are generated with the format `TF-{kind}-{4x4-Crockford}`
//! (BACKEND.md §8.5). Plaintext is returned ONCE; we persist
//! `argon2id(plaintext)` so a DB dump cannot resurrect a key.

use common::auth::backup_codes::hash_one; // same argon2-keyed-pepper hasher
use common::ids::{EnrollmentId, LicenseId, OrderId, ProductId, UserId};
use rand::rngs::OsRng;
use rand::RngCore;
use secrecy::SecretString;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// Crockford base32 (no `I`/`L`/`O`/`U`).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Debug, Clone)]
pub struct Enrollment {
    pub id: EnrollmentId,
    pub user_id: UserId,
    pub product_id: ProductId,
    pub source: String,
    pub source_ref_id: Option<Uuid>,
    pub active: bool,
    pub started_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct License {
    pub id: LicenseId,
    pub user_id: UserId,
    pub product_id: ProductId,
    pub license_key_prefix: String,
    pub source: String,
    pub source_ref_id: Option<Uuid>,
    pub active: bool,
    pub issued_at: OffsetDateTime,
}

#[derive(Clone)]
pub struct EnrollmentsRepo {
    pool: PgPool,
}

#[derive(Clone)]
pub struct LicensesRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum EntitlementsError {
    #[error("hashing failed: {0}")]
    Hash(String),
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl EnrollmentsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or recognize an existing enrollment for `(user, product)`.
    /// The unique index `enrollments_user_product_idx` makes the second
    /// call a no-op (we use `ON CONFLICT DO NOTHING` and tolerate the
    /// `0 rows affected` outcome — the row already exists from a prior
    /// retry of the same webhook).
    pub async fn create_for_purchase_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        user_id: UserId,
        product_id: ProductId,
        order_id: OrderId,
    ) -> Result<(), EntitlementsError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO enrollments (id, user_id, product_id, source, source_ref_id)
            VALUES ($1, $2, $3, 'purchase', $4)
            ON CONFLICT (user_id, product_id) DO NOTHING
            "#,
            id,
            user_id.as_uuid(),
            product_id.as_uuid(),
            order_id.as_uuid(),
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Revoke every enrollment whose `source_ref_id` matches the refunded
    /// order. Used by PR #15's `charge.refunded` handler — purchase-source
    /// enrollments are revoked when the parent order is refunded.
    /// Idempotent: a re-drive marks already-inactive rows as still
    /// inactive (UPDATE matches 0 rows on a second pass).
    pub async fn revoke_for_order_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        order_id: OrderId,
    ) -> Result<u64, EntitlementsError> {
        let res = sqlx::query!(
            r#"
            UPDATE enrollments
            SET active = FALSE,
                completed_at = COALESCE(completed_at, now())
            WHERE source = 'purchase'
              AND source_ref_id = $1
              AND active = TRUE
            "#,
            order_id.as_uuid(),
        )
        .execute(&mut **tx)
        .await?;
        Ok(res.rows_affected())
    }

    /// Revoke every `source = 'subscription'` enrollment for the user.
    /// Used by the `customer.subscription.deleted` handler — purchase-
    /// source enrollments are intentionally NOT revoked (BACKEND.md §8.5).
    pub async fn revoke_subscription_source_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        user_id: UserId,
    ) -> Result<u64, EntitlementsError> {
        let res = sqlx::query!(
            r#"
            UPDATE enrollments
            SET active = FALSE,
                completed_at = COALESCE(completed_at, now())
            WHERE user_id = $1
              AND source = 'subscription'
              AND active = TRUE
            "#,
            user_id.as_uuid()
        )
        .execute(&mut **tx)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn find_for_user_and_product(
        &self,
        user_id: UserId,
        product_id: ProductId,
    ) -> Result<Option<Enrollment>, EntitlementsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, product_id, source, source_ref_id, active, started_at
            FROM enrollments
            WHERE user_id = $1 AND product_id = $2
            "#,
            user_id.as_uuid(),
            product_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Enrollment {
            id: EnrollmentId::from_uuid(r.id),
            user_id: UserId::from_uuid(r.user_id),
            product_id: ProductId::from_uuid(r.product_id),
            source: r.source,
            source_ref_id: r.source_ref_id,
            active: r.active,
            started_at: r.started_at,
        }))
    }

    /// Recompute and persist a progress percentage + last-lesson cursor.
    /// Idempotent: calling twice with the same args is a no-op.
    pub async fn update_progress(
        &self,
        enrollment_id: EnrollmentId,
        progress_pct: i32,
        last_lesson_id: Option<&str>,
    ) -> Result<(), EntitlementsError> {
        sqlx::query!(
            r#"
            UPDATE enrollments
            SET progress_pct = $2,
                last_lesson_id = $3,
                completed_at = CASE WHEN $2 >= 100 AND completed_at IS NULL THEN now() ELSE completed_at END
            WHERE id = $1
            "#,
            enrollment_id.as_uuid(),
            progress_pct,
            last_lesson_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Enrollment>, EntitlementsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, product_id, source, source_ref_id, active, started_at
            FROM enrollments
            WHERE user_id = $1 AND active = TRUE
            ORDER BY started_at DESC
            "#,
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Enrollment {
                id: EnrollmentId::from_uuid(r.id),
                user_id: UserId::from_uuid(r.user_id),
                product_id: ProductId::from_uuid(r.product_id),
                source: r.source,
                source_ref_id: r.source_ref_id,
                active: r.active,
                started_at: r.started_at,
            })
            .collect())
    }
}

/// A freshly-issued license that includes the plaintext key (returned ONCE
/// to the webhook handler so it can be surfaced to the user / receipt
/// email).
pub struct IssuedLicense {
    pub id: LicenseId,
    pub plaintext: SecretString,
    pub prefix: String,
}

impl LicensesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a license key, hash it, persist it inside `tx`, return the
    /// plaintext. BACKEND.md §8.5: only the hash + first-8-char prefix
    /// land in the DB.
    ///
    /// Idempotent on the `(license_key_hash UNIQUE)` constraint — a
    /// re-driven webhook for the same order generates a NEW random key
    /// but the **previous** key row already exists; we skip insert when
    /// an active license row for `(user, product, source_ref_id)` is
    /// already present. The CHECK is done before generating to avoid
    /// burning Argon2 cycles on a no-op.
    pub async fn issue_for_purchase_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        pepper: &SecretString,
        user_id: UserId,
        product_id: ProductId,
        order_id: OrderId,
        kind_prefix: &str,
    ) -> Result<Option<IssuedLicense>, EntitlementsError> {
        // Skip if we already issued one for this purchase.
        let existing = sqlx::query!(
            r#"
            SELECT id FROM licenses
            WHERE user_id = $1
              AND product_id = $2
              AND source_ref_id = $3
              AND active = TRUE
            "#,
            user_id.as_uuid(),
            product_id.as_uuid(),
            order_id.as_uuid(),
        )
        .fetch_optional(&mut **tx)
        .await?;
        if existing.is_some() {
            return Ok(None);
        }

        let plaintext = generate_license_key(kind_prefix);
        let prefix: String = plaintext.chars().take(8).collect();
        let hash_bytes = hash_one(SecretString::from(plaintext.clone()), pepper.clone())
            .await
            .map_err(|e| EntitlementsError::Hash(e.to_string()))?;
        // sha256 → BYTEA UNIQUE is the column shape. The hash from
        // `backup_codes::hash_one` returns the full PHC string; for
        // license keys we use the hash as a BYTEA. The column is BYTEA
        // UNIQUE so we just pass the bytes through.
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO licenses (
                id, user_id, product_id,
                license_key_hash, license_key_prefix,
                source, source_ref_id
            ) VALUES (
                $1, $2, $3,
                $4, $5,
                'purchase', $6
            )
            "#,
            id,
            user_id.as_uuid(),
            product_id.as_uuid(),
            &hash_bytes[..],
            prefix,
            order_id.as_uuid(),
        )
        .execute(&mut **tx)
        .await?;
        Ok(Some(IssuedLicense {
            id: LicenseId::from_uuid(id),
            plaintext: SecretString::from(plaintext),
            prefix,
        }))
    }

    /// Revoke every license whose `source_ref_id` matches the refunded
    /// order. Sets `active = FALSE` + `revoked_at = now()` (the schema
    /// CHECK requires both). Idempotent.
    pub async fn revoke_for_order_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        order_id: OrderId,
    ) -> Result<u64, EntitlementsError> {
        let res = sqlx::query!(
            r#"
            UPDATE licenses
            SET active = FALSE,
                revoked_at = now()
            WHERE source = 'purchase'
              AND source_ref_id = $1
              AND active = TRUE
            "#,
            order_id.as_uuid(),
        )
        .execute(&mut **tx)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn list_active_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<License>, EntitlementsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, product_id, license_key_prefix, source,
                   source_ref_id, active, issued_at
            FROM licenses
            WHERE user_id = $1 AND active = TRUE
            ORDER BY issued_at DESC
            "#,
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| License {
                id: LicenseId::from_uuid(r.id),
                user_id: UserId::from_uuid(r.user_id),
                product_id: ProductId::from_uuid(r.product_id),
                license_key_prefix: r.license_key_prefix,
                source: r.source,
                source_ref_id: r.source_ref_id,
                active: r.active,
                issued_at: r.issued_at,
            })
            .collect())
    }
}

/// `TF-{kind}-{4x4-Crockford}` per BACKEND.md §8.5. 16 random bytes →
/// 4 groups of 4 Crockford chars (each char encodes 5 bits, so 4 groups
/// of 4 = 80 bits of entropy from the 16-byte source — using modulo, which
/// has tiny bias but is fine at this entropy level for a one-shot
/// out-of-band secret).
pub fn generate_license_key(kind_prefix: &str) -> String {
    let mut buf = [0u8; 16];
    OsRng.fill_bytes(&mut buf);
    let mut chars: Vec<char> = buf
        .iter()
        .map(|b| CROCKFORD[(*b as usize) % 32] as char)
        .collect();
    // Format as 4-4-4-4 groups.
    let mut grouped = String::with_capacity(19);
    for (i, c) in chars.drain(..).enumerate() {
        if i > 0 && i % 4 == 0 {
            grouped.push('-');
        }
        grouped.push(c);
    }
    format!("TF-{}-{grouped}", kind_prefix.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_key_shape() {
        for _ in 0..1000 {
            let k = generate_license_key("rr");
            // "TF-RR-XXXX-XXXX-XXXX-XXXX" = 3+2+1+(4+1)*3+4 = … verify length.
            // Actually: "TF-RR-XXXX-XXXX-XXXX-XXXX" = 25 chars.
            assert_eq!(k.len(), 25, "{k}");
            assert!(k.starts_with("TF-RR-"));
            for c in k.chars().filter(|c| *c != '-') {
                assert!(
                    CROCKFORD.contains(&(c as u8)),
                    "{k} has non-Crockford char {c}"
                );
                // Disallow the visually-ambiguous letters.
                assert!(!"ILOU".contains(c), "{k} has ambiguous char {c}");
            }
        }
    }

    #[test]
    fn license_keys_are_unique_across_many_generations() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..10_000 {
            assert!(seen.insert(generate_license_key("rr")));
        }
    }
}
