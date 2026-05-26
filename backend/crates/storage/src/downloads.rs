//! `DownloadsCatalogRepo` + `DownloadGrantsRepo`.
//!
//! BACKEND.md §4 0016, §11 (indicators/downloads surface).
//!
//! The catalog is a public-ish list of artifacts keyed by `(product_id,
//! platform, version)` with an R2 object key, SHA-256, and byte size. The
//! grants table records that a specific user is permitted to download a
//! specific catalog row and tracks usage (last_downloaded_at + count).
//!
//! Grants are created implicitly the first time an entitled user requests
//! a presigned URL — we don't pre-grant on enrollment because the catalog
//! grows independently of licenses (a new platform binary becomes
//! available without re-touching the entitlement row).

use common::ids::{ProductId, UserId};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DownloadCatalogEntry {
    pub id: Uuid,
    pub product_id: ProductId,
    pub platform: String,
    pub version: String,
    pub file_r2_key: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub released_at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct DownloadsCatalogRepo {
    pool: PgPool,
}

impl DownloadsCatalogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Catalog rows for a single product, newest first.
    pub async fn list_for_product(
        &self,
        product_id: ProductId,
    ) -> Result<Vec<DownloadCatalogEntry>, DownloadsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, product_id, platform, version, file_r2_key,
                   sha256, size_bytes, released_at
            FROM downloads_catalog
            WHERE product_id = $1
            ORDER BY released_at DESC
            "#,
            product_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| DownloadCatalogEntry {
                id: r.id,
                product_id: ProductId::from_uuid(r.product_id),
                platform: r.platform,
                version: r.version,
                file_r2_key: r.file_r2_key,
                sha256: r.sha256,
                size_bytes: r.size_bytes,
                released_at: r.released_at,
            })
            .collect())
    }

    /// All catalog rows the user is entitled to — joined against licenses.
    /// One row per (license, catalog-entry) pair. The handler dedupes/groups.
    pub async fn list_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DownloadCatalogEntry>, DownloadsError> {
        let rows = sqlx::query!(
            r#"
            SELECT dc.id, dc.product_id, dc.platform, dc.version,
                   dc.file_r2_key, dc.sha256, dc.size_bytes, dc.released_at
            FROM downloads_catalog dc
            WHERE dc.product_id IN (
                SELECT DISTINCT l.product_id
                FROM licenses l
                WHERE l.user_id = $1 AND l.active = TRUE
            )
            ORDER BY dc.product_id, dc.released_at DESC
            "#,
            user_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| DownloadCatalogEntry {
                id: r.id,
                product_id: ProductId::from_uuid(r.product_id),
                platform: r.platform,
                version: r.version,
                file_r2_key: r.file_r2_key,
                sha256: r.sha256,
                size_bytes: r.size_bytes,
                released_at: r.released_at,
            })
            .collect())
    }

    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<DownloadCatalogEntry>, DownloadsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, product_id, platform, version, file_r2_key,
                   sha256, size_bytes, released_at
            FROM downloads_catalog
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| DownloadCatalogEntry {
            id: r.id,
            product_id: ProductId::from_uuid(r.product_id),
            platform: r.platform,
            version: r.version,
            file_r2_key: r.file_r2_key,
            sha256: r.sha256,
            size_bytes: r.size_bytes,
            released_at: r.released_at,
        }))
    }

    /// Insert a catalog row idempotently on `(product_id, platform,
    /// version)`. Used by admin/CRUD + seeders. PR #11 ships a thin path;
    /// the admin CRUD UI lives in PR #14.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        &self,
        product_id: ProductId,
        platform: &str,
        version: &str,
        file_r2_key: &str,
        sha256: &str,
        size_bytes: i64,
    ) -> Result<Uuid, DownloadsError> {
        let id = Uuid::now_v7();
        let row = sqlx::query!(
            r#"
            INSERT INTO downloads_catalog (
                id, product_id, platform, version, file_r2_key, sha256, size_bytes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (product_id, platform, version) DO UPDATE
            SET file_r2_key = EXCLUDED.file_r2_key,
                sha256 = EXCLUDED.sha256,
                size_bytes = EXCLUDED.size_bytes
            RETURNING id
            "#,
            id,
            product_id.as_uuid(),
            platform,
            version,
            file_r2_key,
            sha256,
            size_bytes,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.id)
    }
}

#[derive(Clone, Default)]
pub struct DownloadGrantsRepo {
    // Repo currently only holds tx-scoped methods (the grant bump is
    // always paired with an audit row, written in the caller's tx). A
    // `PgPool` field will land here when the dashboard surfaces a "your
    // download history" read endpoint that doesn't ride someone else's tx.
}

#[derive(Debug, Clone)]
pub struct DownloadGrant {
    pub id: Uuid,
    pub user_id: UserId,
    pub download_id: Uuid,
    pub granted_at: OffsetDateTime,
    pub last_downloaded_at: Option<OffsetDateTime>,
    pub download_count: i32,
}

impl DownloadGrantsRepo {
    pub fn new(_pool: PgPool) -> Self {
        Self::default()
    }

    /// Record a download access — UPSERTs the grant row and bumps the
    /// counter atomically. Called from the presign endpoint so the audit
    /// row + counter bump live in the same transaction as minting the URL
    /// (BACKEND.md §22 rule 8 — audit travels with the action).
    pub async fn record_access_in_tx<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        user_id: UserId,
        download_id: Uuid,
    ) -> Result<DownloadGrant, DownloadsError> {
        let id = Uuid::now_v7();
        let row = sqlx::query!(
            r#"
            INSERT INTO download_grants (id, user_id, download_id, last_downloaded_at, download_count)
            VALUES ($1, $2, $3, now(), 1)
            ON CONFLICT (user_id, download_id) DO UPDATE
            SET last_downloaded_at = now(),
                download_count = download_grants.download_count + 1
            RETURNING id, user_id, download_id, granted_at, last_downloaded_at, download_count
            "#,
            id,
            user_id.as_uuid(),
            download_id,
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(DownloadGrant {
            id: row.id,
            user_id: UserId::from_uuid(row.user_id),
            download_id: row.download_id,
            granted_at: row.granted_at,
            last_downloaded_at: row.last_downloaded_at,
            download_count: row.download_count,
        })
    }
}
