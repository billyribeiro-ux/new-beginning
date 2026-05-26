//! PR #10 evidence test.
//!
//! End-to-end exercise of the billing-invoice surface without the HTTP
//! transport: insert a user + invoice via the repos, mark the PDF as
//! present in the recording R2, and verify:
//!   * `list_for_user` returns the row with `pdf_r2_key.is_some()`.
//!   * `find_for_user` scopes correctly (wrong user → None).
//!   * `attach_pdf_r2_key` is idempotent and never overwrites a foreign key.
//!   * The presign helper returns a URL containing the key.
//!
//! Requires a Postgres at `DATABASE_URL` (defaults to the dev compose).

use bytes::Bytes;
use r2_client::{ObjectStore, RecordingObjectStore};
use sqlx::PgPool;
use std::time::Duration;
use storage::{InvoicesRepo, UsersRepo};
use time::OffsetDateTime;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("test DB pool — is `just up` running?")
}

fn unique_email(tag: &str) -> String {
    use uuid::Uuid;
    format!("pr10-{tag}-{}@example.test", Uuid::new_v4())
}

#[tokio::test]
async fn full_pr10_flow_storage_plus_recording_r2() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let invoices = InvoicesRepo::new(pool.clone());

    let user_a = users
        .create(&unique_email("a"), "Alice", "fake-hash-not-used-here")
        .await
        .unwrap();
    let user_b = users
        .create(&unique_email("b"), "Bob", "fake-hash-not-used-here")
        .await
        .unwrap();

    // Insert an invoice for user_a via the same `upsert_in_tx` path the
    // webhook uses. Exercise twice to prove idempotency on stripe_invoice_id.
    let stripe_invoice_id = format!("in_test_{}", uuid::Uuid::new_v4());
    let date = OffsetDateTime::now_utc();

    let mut tx = pool.begin().await.unwrap();
    invoices
        .upsert_in_tx(
            &mut tx,
            None,
            user_a.id,
            &stripe_invoice_id,
            "INV-PR10-001",
            "paid",
            12_345,
            "usd",
            date,
        )
        .await
        .unwrap();
    invoices
        .upsert_in_tx(
            &mut tx,
            None,
            user_a.id,
            &stripe_invoice_id,
            "INV-PR10-001",
            "paid",
            12_345,
            "usd",
            date,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // The user has exactly one invoice; pdf_r2_key is still NULL.
    let list = invoices.list_for_user(user_a.id).await.unwrap();
    let only_one = list
        .iter()
        .filter(|r| r.stripe_invoice_id == stripe_invoice_id)
        .count();
    assert_eq!(
        only_one, 1,
        "upsert should be idempotent on stripe_invoice_id"
    );
    let row = list
        .into_iter()
        .find(|r| r.stripe_invoice_id == stripe_invoice_id)
        .unwrap();
    assert!(row.pdf_r2_key.is_none());
    let invoice_id = row.id;

    // Worker side: put the PDF into the recording R2 + attach the key.
    let r2 = RecordingObjectStore::new();
    let key = format!("invoices/{}/{}.pdf", user_a.id, invoice_id);
    r2.put_object(&key, Bytes::from_static(b"%PDF-fake"), "application/pdf")
        .await
        .unwrap();
    let n = invoices
        .attach_pdf_r2_key(&stripe_invoice_id, &key)
        .await
        .unwrap();
    assert_eq!(n, 1, "first attach updates exactly one row");

    // Re-running with the SAME key is idempotent (still matches, still 1
    // row "updated" because the WHERE matches; we don't care about the
    // count, just that no error and the value is unchanged).
    invoices
        .attach_pdf_r2_key(&stripe_invoice_id, &key)
        .await
        .unwrap();
    let after_reattach = invoices
        .find_for_user(user_a.id, invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_reattach.pdf_r2_key.as_deref(), Some(key.as_str()));

    // Attempting to overwrite with a DIFFERENT key MUST NOT mutate the row.
    let n = invoices
        .attach_pdf_r2_key(&stripe_invoice_id, "invoices/SOMEONE_ELSE/zzz.pdf")
        .await
        .unwrap();
    assert_eq!(n, 0, "different key must not overwrite an existing one");
    let after_conflict = invoices
        .find_for_user(user_a.id, invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_conflict.pdf_r2_key.as_deref(), Some(key.as_str()));

    // Entitlement: user_b cannot resolve user_a's invoice.
    let cross_user = invoices.find_for_user(user_b.id, invoice_id).await.unwrap();
    assert!(
        cross_user.is_none(),
        "find_for_user must be scoped by user_id — entitlement check"
    );

    // Presign mints a URL referencing the key.
    let url = r2.presigned_get(&key, Duration::from_secs(300)).unwrap();
    assert!(url.as_str().contains(&key));
    assert!(url.as_str().contains("ttl=300"));

    // HEAD on the recording store returns Ok for present keys.
    r2.head_object(&key).await.unwrap();

    // HEAD on a missing key surfaces NotFound (handler maps this → 404).
    let missing = r2.head_object("invoices/nope/missing.pdf").await;
    assert!(matches!(
        missing.unwrap_err(),
        r2_client::StoreError::NotFound
    ));
}
