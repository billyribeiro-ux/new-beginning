//! PR #15 evidence test — refund flow at the storage layer.
//!
//! Walks: paid order with license + enrollment + invoice → `mark_refunded_in_tx`
//! flips status, `revoke_for_order_in_tx` (licenses + enrollments) flips
//! active flags, audit row records the action.
//!
//! Also asserts idempotency: a second pass through the same flow is a
//! no-op (the WHERE clauses gate everything to `status='paid'` / `active=TRUE`).

use common::ids::OrderId;
use secrecy::SecretString;
use sqlx::PgPool;
use storage::{
    AuditRepo, EnrollmentsRepo, InvoicesRepo, LicensesRepo, OrdersRepo, ProductsRepo, UsersRepo,
};
use time::OffsetDateTime;
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
    format!("pr15-{tag}-{}@example.test", Uuid::new_v4())
}

async fn insert_indicator_product(pool: &PgPool, slug: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO products (
            id, slug, kind, name, tagline, description, price_cents,
            highlights, deliverables, requirements, media_poster_color,
            media_accent
        ) VALUES (
            $1, $2, 'indicator', 'PR15 Indicator', 'tag', 'desc', 9900,
            '{}', '{}', '{}', '#000', '#fff'
        )
        "#,
        id,
        slug,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn insert_paid_order(pool: &PgPool, user_id: common::ids::UserId) -> OrderId {
    let id = OrderId::new();
    let pi = format!("pi_pr15_{}", Uuid::new_v4().simple());
    sqlx::query!(
        r#"
        INSERT INTO orders (
            id, user_id, status, subtotal_cents, tax_cents, total_cents,
            currency, stripe_payment_intent_id, paid_at, cart_snapshot
        ) VALUES (
            $1, $2, 'paid', 9900, 0, 9900, 'usd', $3, now(), '[]'::jsonb
        )
        "#,
        id.as_uuid(),
        user_id.as_uuid(),
        pi,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn payment_intent_for_order(pool: &PgPool, order_id: OrderId) -> String {
    sqlx::query!(
        "SELECT stripe_payment_intent_id FROM orders WHERE id = $1",
        order_id.as_uuid()
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .stripe_payment_intent_id
    .expect("test order must have payment_intent")
}

#[tokio::test]
async fn refund_revokes_entitlements_and_idempotent_redrive() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let products = ProductsRepo::new(pool.clone());
    let orders = OrdersRepo::new(pool.clone());
    let licenses = LicensesRepo::new(pool.clone());
    let enrollments = EnrollmentsRepo::new(pool.clone());
    let invoices = InvoicesRepo::new(pool.clone());
    let audit = AuditRepo::new(pool.clone());

    let alice = users
        .create(&unique_email("alice"), "Alice", "x")
        .await
        .unwrap();

    let slug = format!("ind-pr15-{}", Uuid::new_v4().simple());
    insert_indicator_product(&pool, &slug).await;
    let product = products.find_by_slug(&slug).await.unwrap().unwrap();

    let order_id = insert_paid_order(&pool, alice.id).await;
    let pi = payment_intent_for_order(&pool, order_id).await;

    // Issue entitlements as if from the checkout.session.completed handler.
    let pepper = SecretString::from("test-pepper".to_string());
    let mut tx = pool.begin().await.unwrap();
    let issued = licenses
        .issue_for_purchase_in_tx(&mut tx, &pepper, alice.id, product.id, order_id, "IN")
        .await
        .unwrap()
        .unwrap();
    enrollments
        .create_for_purchase_in_tx(&mut tx, alice.id, product.id, order_id)
        .await
        .unwrap();
    invoices
        .upsert_in_tx(
            &mut tx,
            Some(order_id),
            alice.id,
            &format!("in_test_pr15_{}", Uuid::new_v4()),
            "INV-PR15",
            "paid",
            9_900,
            "usd",
            OffsetDateTime::now_utc(),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Sanity: license + enrollment are active.
    let active = licenses.list_active_for_user(alice.id).await.unwrap();
    assert!(active.iter().any(|l| l.id == issued.id));
    let enr = enrollments
        .find_for_user_and_product(alice.id, product.id)
        .await
        .unwrap()
        .unwrap();
    assert!(enr.active);

    // Refund flow — same sequence the `charge.refunded` dispatcher runs.
    let mut tx = pool.begin().await.unwrap();
    let marked = orders
        .mark_refunded_in_tx(&mut tx, &pi)
        .await
        .unwrap()
        .expect("first refund must match the paid order");
    assert_eq!(marked.0, order_id);
    assert_eq!(marked.1, alice.id);

    let licenses_revoked = licenses
        .revoke_for_order_in_tx(&mut tx, order_id)
        .await
        .unwrap();
    let enrollments_revoked = enrollments
        .revoke_for_order_in_tx(&mut tx, order_id)
        .await
        .unwrap();
    audit
        .record_in_tx(
            &mut tx,
            None,
            "order.refunded",
            "orders",
            &order_id.to_string(),
            serde_json::json!({
                "payment_intent_id": pi,
                "licenses_revoked": licenses_revoked,
                "enrollments_revoked": enrollments_revoked,
            }),
            None,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(licenses_revoked, 1);
    assert_eq!(enrollments_revoked, 1);

    // License + enrollment are now inactive.
    let still_active = licenses.list_active_for_user(alice.id).await.unwrap();
    assert!(
        !still_active.iter().any(|l| l.id == issued.id),
        "refunded license must drop out of active list"
    );
    let enr = enrollments
        .find_for_user_and_product(alice.id, product.id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        !enr.active,
        "refunded purchase-source enrollment must be inactive"
    );

    // Order is in 'refunded'.
    let order = orders.find_by_id(order_id).await.unwrap().unwrap();
    assert_eq!(order.status, "refunded");

    // Audit row exists with the right action.
    let trail = audit
        .list_for_target("orders", &order_id.to_string())
        .await
        .unwrap();
    assert!(trail.iter().any(|r| r.action == "order.refunded"));

    // Idempotent re-drive: second pass matches 0 rows everywhere.
    let mut tx = pool.begin().await.unwrap();
    let second = orders.mark_refunded_in_tx(&mut tx, &pi).await.unwrap();
    assert!(
        second.is_none(),
        "second mark_refunded MUST be a no-op (status != 'paid' guard)"
    );
    let again_licenses = licenses
        .revoke_for_order_in_tx(&mut tx, order_id)
        .await
        .unwrap();
    let again_enrollments = enrollments
        .revoke_for_order_in_tx(&mut tx, order_id)
        .await
        .unwrap();
    assert_eq!(again_licenses, 0);
    assert_eq!(again_enrollments, 0);
    tx.commit().await.unwrap();
}
