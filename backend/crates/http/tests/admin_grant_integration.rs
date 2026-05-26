//! PR #14 evidence test — admin manual grant flow at the storage layer.
//!
//! Walks the same `issue_for_purchase_in_tx` plus `create_for_purchase_in_tx`
//! plus `audit::record_in_tx` sequence the `/v1/admin/customers/{id}/grant-entitlement` handler runs.
//!
//! Asserts these properties:
//! * The grant produces an active license with a known prefix.
//! * Course-kind grants produce both a license AND an enrollment row.
//! * The audit row carries the reason + product slug for traceability.
//! * Repeating the grant with a fresh synthetic order id mints a NEW
//!   license (manual re-issuance is allowed by design).

use common::ids::OrderId;
use secrecy::SecretString;
use sqlx::PgPool;
use storage::{AuditRepo, EnrollmentsRepo, LicensesRepo, ProductsRepo, UsersRepo};
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
    format!("pr14-{tag}-{}@example.test", Uuid::new_v4())
}

async fn insert_course_product(pool: &PgPool, slug: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO products (
            id, slug, kind, name, tagline, description, price_cents,
            highlights, deliverables, requirements, media_poster_color,
            media_accent
        ) VALUES (
            $1, $2, 'course', 'PR14 Course', 'tag', 'desc', 29900,
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

#[tokio::test]
async fn admin_grant_writes_license_enrollment_and_audit() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let products = ProductsRepo::new(pool.clone());
    let licenses = LicensesRepo::new(pool.clone());
    let enrollments = EnrollmentsRepo::new(pool.clone());
    let audit = AuditRepo::new(pool.clone());

    let alice = users
        .create(&unique_email("alice"), "Alice", "x")
        .await
        .unwrap();
    let admin_actor = users
        .create(&unique_email("admin"), "Admin", "x")
        .await
        .unwrap();

    let slug = format!("course-pr14-{}", Uuid::new_v4().simple());
    insert_course_product(&pool, &slug).await;
    let product = products.find_by_slug(&slug).await.unwrap().unwrap();

    // Manual grant tx — exact sequence the customers::grant_entitlement
    // handler runs.
    let pepper = SecretString::from("admin-grant-test-pepper".to_string());
    let synthetic_order_id = OrderId::new();
    let reason = "support ticket #4242 — payment gateway timed out twice";

    let mut tx = pool.begin().await.unwrap();
    let issued = licenses
        .issue_for_purchase_in_tx(
            &mut tx,
            &pepper,
            alice.id,
            product.id,
            synthetic_order_id,
            "CR",
        )
        .await
        .unwrap()
        .expect("first grant must mint a license");
    enrollments
        .create_for_purchase_in_tx(&mut tx, alice.id, product.id, synthetic_order_id)
        .await
        .unwrap();
    audit
        .record_in_tx(
            &mut tx,
            Some(admin_actor.id),
            "admin.entitlement.granted",
            "users",
            &alice.id.to_string(),
            serde_json::json!({
                "product_id": product.id.to_string(),
                "product_slug": product.slug,
                "kind": product.kind,
                "license_prefix": issued.prefix,
                "synthetic_order_id": synthetic_order_id.to_string(),
                "reason": reason,
            }),
            None,
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // License is active + carries the expected prefix shape.
    assert!(issued.prefix.starts_with("TF-CR-"));
    let active = licenses.list_active_for_user(alice.id).await.unwrap();
    assert!(active.iter().any(|l| l.id == issued.id));

    // Enrollment row exists.
    let enr = enrollments
        .find_for_user_and_product(alice.id, product.id)
        .await
        .unwrap()
        .expect("course-kind grant must produce an enrollment");
    assert!(enr.active);

    // Audit trail records the actor + reason.
    let trail = audit
        .list_for_target("users", &alice.id.to_string())
        .await
        .unwrap();
    let grant_row = trail
        .iter()
        .find(|r| r.action == "admin.entitlement.granted")
        .expect("audit row must exist");
    assert_eq!(grant_row.actor_user_id, Some(admin_actor.id));
    assert_eq!(
        grant_row.metadata.get("reason").and_then(|v| v.as_str()),
        Some(reason)
    );
    assert_eq!(
        grant_row
            .metadata
            .get("product_slug")
            .and_then(|v| v.as_str()),
        Some(product.slug.as_str())
    );

    // Re-grant with a FRESH synthetic order id MUST mint a new license
    // (manual re-issuance is allowed). The skip-existing check keys off
    // `(user, product, source_ref_id)`, not just `(user, product)`.
    let synthetic_2 = OrderId::new();
    let mut tx2 = pool.begin().await.unwrap();
    let issued2 = licenses
        .issue_for_purchase_in_tx(&mut tx2, &pepper, alice.id, product.id, synthetic_2, "CR")
        .await
        .unwrap()
        .expect("re-grant with fresh ref id must mint a new license");
    tx2.commit().await.unwrap();
    assert_ne!(
        issued.id, issued2.id,
        "re-grant must produce a distinct license"
    );
}
