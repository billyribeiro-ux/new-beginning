//! PR #11 worker-job evidence: `fetch_invoice_pdfs` driver.
//!
//! Exercises the storage selection: `InvoicesRepo::list_missing_pdf`
//! returns rows whose `pdf_r2_key IS NULL` and skips already-attached
//! ones. The full worker loop (Stripe GET → HTTP fetch → R2 PUT → DB
//! UPDATE) is integration-tested via the recording fakes in
//! `pr-11-worker.md` evidence (a real network fetch from stripe.test
//! would 404). This test verifies the storage selection that drives it.

use sqlx::PgPool;
use storage::{InvoicesRepo, UsersRepo};
use time::OffsetDateTime;
use uuid::Uuid;

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
    format!("pr11w-{tag}-{}@example.test", Uuid::new_v4())
}

#[tokio::test]
async fn list_missing_pdf_returns_only_null_keys() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let invoices = InvoicesRepo::new(pool.clone());
    let user = users
        .create(&unique_email("u"), "Worker User", "x")
        .await
        .unwrap();

    let stripe_with_pdf = format!("in_pr11_with_{}", Uuid::new_v4());
    let stripe_without_pdf = format!("in_pr11_without_{}", Uuid::new_v4());
    let date = OffsetDateTime::now_utc();

    let mut tx = pool.begin().await.unwrap();
    invoices
        .upsert_in_tx(
            &mut tx,
            None,
            user.id,
            &stripe_with_pdf,
            "INV-PR11-W",
            "paid",
            10_000,
            "usd",
            date,
        )
        .await
        .unwrap();
    invoices
        .upsert_in_tx(
            &mut tx,
            None,
            user.id,
            &stripe_without_pdf,
            "INV-PR11-NW",
            "paid",
            20_000,
            "usd",
            date,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Stamp one of them as already-attached (simulating a successful
    // prior worker run).
    let key = "invoices/x/y.pdf";
    let n = invoices
        .attach_pdf_r2_key(&stripe_with_pdf, key)
        .await
        .unwrap();
    assert_eq!(n, 1);

    // list_missing_pdf must return ONLY the one without a key. Filter by
    // our two ids in case other test rows lurk in the DB.
    let missing: std::collections::HashSet<String> = invoices
        .list_missing_pdf(1000)
        .await
        .unwrap()
        .into_iter()
        .map(|(_, _, sid)| sid)
        .collect();
    assert!(
        !missing.contains(&stripe_with_pdf),
        "row with key already attached must NOT appear in list_missing_pdf"
    );
    assert!(
        missing.contains(&stripe_without_pdf),
        "row without a key MUST appear in list_missing_pdf"
    );
}
