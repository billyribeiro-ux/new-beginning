//! PR #11 evidence test.
//!
//! Walks the indicators + downloads surface end-to-end at the storage +
//! repo layer:
//!   * Create user + product + license + downloads_catalog row.
//!   * `list_active_for_user(user)` returns the license.
//!   * `downloads_catalog.list_for_user(user)` returns the catalog row
//!     because the user holds a license for that product.
//!   * `downloads_catalog.list_for_user(other_user)` is empty.
//!   * `download_grants.record_access_in_tx` bumps count atomically; the
//!     second call from a separate tx increments to 2.
//!   * AuditRepo records the action in the same tx as the grant bump.

use bytes::Bytes;
use common::ids::OrderId;
use r2_client::{ObjectStore, RecordingObjectStore};
use secrecy::SecretString;
use sqlx::PgPool;
use std::time::Duration;
use storage::{
    AuditRepo, DownloadGrantsRepo, DownloadsCatalogRepo, LicensesRepo, ProductsRepo, UsersRepo,
};
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("test DB pool — is `just up` running?")
}

fn unique_email(tag: &str) -> String {
    format!("pr11-{tag}-{}@example.test", Uuid::new_v4())
}

async fn insert_product(pool: &PgPool, slug: &str, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO products (
            id, slug, kind, name, tagline, description, price_cents,
            highlights, deliverables, requirements, media_poster_color,
            media_accent
        ) VALUES (
            $1, $2, 'indicator', $3, 'tag', 'desc', 9900,
            '{}', '{}', '{}', '#000', '#fff'
        )
        "#,
        id,
        slug,
        name,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn pr11_indicators_and_downloads_flow() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let products = ProductsRepo::new(pool.clone());
    let licenses = LicensesRepo::new(pool.clone());
    let dl_catalog = DownloadsCatalogRepo::new(pool.clone());
    let dl_grants = DownloadGrantsRepo::new(pool.clone());
    let audit = AuditRepo::new(pool.clone());

    let alice = users
        .create(&unique_email("alice"), "Alice", "hash-not-used")
        .await
        .unwrap();
    let bob = users
        .create(&unique_email("bob"), "Bob", "hash-not-used")
        .await
        .unwrap();

    let slug = format!("ind-pr11-{}", Uuid::new_v4().simple());
    let product_id_raw = insert_product(&pool, &slug, "Indicator PR11").await;
    let product = products.find_by_slug(&slug).await.unwrap().unwrap();
    assert_eq!(product.id.as_uuid(), product_id_raw);

    // Issue a license for Alice via the same tx-scoped path the webhook
    // dispatcher uses. The pepper here is irrelevant to the test (it's
    // not verified anywhere) — any SecretString works.
    let pepper = SecretString::from("pepper-for-pr11-test".to_string());
    let order_id = OrderId::new();
    let mut tx = pool.begin().await.unwrap();
    let issued = licenses
        .issue_for_purchase_in_tx(&mut tx, &pepper, alice.id, product.id, order_id, "IN")
        .await
        .unwrap()
        .expect("first issuance should mint a license");
    tx.commit().await.unwrap();
    assert!(issued.prefix.starts_with("TF-IN-"));

    // Insert a downloads_catalog row via the upsert path.
    let dl_id = dl_catalog
        .upsert(
            product.id,
            "macos-arm64",
            "1.0.0",
            &format!("downloads/{}/1.0.0/indicator.zip", product.id),
            &"a".repeat(64),
            123_456,
        )
        .await
        .unwrap();

    // Re-upsert is idempotent (returns the same id semantically — same
    // (product_id, platform, version) key — and overwrites the file_r2_key
    // etc. as part of ON CONFLICT DO UPDATE).
    let dl_id_again = dl_catalog
        .upsert(
            product.id,
            "macos-arm64",
            "1.0.0",
            &format!("downloads/{}/1.0.0/indicator-v2.zip", product.id),
            &"b".repeat(64),
            234_567,
        )
        .await
        .unwrap();
    assert_eq!(
        dl_id, dl_id_again,
        "upsert with same (product, platform, version) must keep id stable"
    );

    // Alice's catalog list includes the row (via her license).
    let alice_catalog = dl_catalog.list_for_user(alice.id).await.unwrap();
    assert!(
        alice_catalog.iter().any(|c| c.id == dl_id),
        "alice should see the catalog row her license entitles her to"
    );

    // Bob's catalog list does NOT include it (no license).
    let bob_catalog = dl_catalog.list_for_user(bob.id).await.unwrap();
    assert!(
        !bob_catalog.iter().any(|c| c.id == dl_id),
        "bob has no license, so the catalog row must not show up"
    );

    // Record an access for Alice in a tx with an audit row. Counter = 1.
    let mut tx = pool.begin().await.unwrap();
    let grant = dl_grants
        .record_access_in_tx(&mut tx, alice.id, dl_id)
        .await
        .unwrap();
    audit
        .record_in_tx(
            &mut tx,
            Some(alice.id),
            "download.url_minted",
            "downloads_catalog",
            &dl_id.to_string(),
            serde_json::json!({ "grant_count": grant.download_count }),
            None,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(grant.download_count, 1);

    // Second access — counter bumps to 2.
    let mut tx = pool.begin().await.unwrap();
    let grant2 = dl_grants
        .record_access_in_tx(&mut tx, alice.id, dl_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(grant2.download_count, 2);

    // Audit trail has at least one row for this download.
    let audit_rows = audit
        .list_for_target("downloads_catalog", &dl_id.to_string())
        .await
        .unwrap();
    assert!(!audit_rows.is_empty(), "audit row must persist");
    assert_eq!(audit_rows[0].action, "download.url_minted");

    // Recording R2: the catalog row's file_r2_key would resolve to a
    // presigned URL via the same trait the handler uses.
    let r2 = RecordingObjectStore::new();
    // Pre-populate as if the worker had uploaded it.
    r2.put_object(
        &format!("downloads/{}/1.0.0/indicator-v2.zip", product.id),
        Bytes::from_static(b"FAKE_BINARY_BYTES"),
        "application/zip",
    )
    .await
    .unwrap();
    let url = r2
        .presigned_get(
            &format!("downloads/{}/1.0.0/indicator-v2.zip", product.id),
            Duration::from_secs(300),
        )
        .unwrap();
    assert!(url.as_str().contains("downloads/"));
    assert!(url.as_str().contains("ttl=300"));
}
