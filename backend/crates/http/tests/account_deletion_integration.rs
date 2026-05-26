//! PR #16 evidence — soft-delete flow.
//!
//! * `find_by_email` returns the user before deletion, None after.
//! * `find_by_id` returns None after deletion (deleted_at gate).
//! * Email is pseudonymized (no PII left on the row).
//! * Repeat soft_delete returns 0 (idempotent).

use sqlx::PgPool;
use storage::UsersRepo;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("test DB pool")
}

fn unique_email(tag: &str) -> String {
    format!("pr16-{tag}-{}@example.test", Uuid::new_v4())
}

#[tokio::test]
async fn soft_delete_pseudonymizes_and_is_idempotent() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let email = unique_email("alice");
    let alice = users.create(&email, "Alice", "x").await.unwrap();

    // Visible pre-deletion.
    assert!(users.find_by_email(&email).await.unwrap().is_some());
    assert!(users.find_by_id(alice.id).await.unwrap().is_some());

    let n = users.soft_delete(alice.id).await.unwrap();
    assert_eq!(n, 1);

    // find_by_email + find_by_id both gate on deleted_at IS NULL.
    assert!(users.find_by_email(&email).await.unwrap().is_none());
    assert!(users.find_by_id(alice.id).await.unwrap().is_none());

    // Pseudonymized email is on the raw row.
    let row = sqlx::query!(
        "SELECT email::text AS \"email!\" FROM users WHERE id = $1",
        alice.id.as_uuid()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(row.email.ends_with("@deleted.tradeflex.invalid"));

    // Idempotent re-delete.
    let again = users.soft_delete(alice.id).await.unwrap();
    assert_eq!(again, 0);
}
