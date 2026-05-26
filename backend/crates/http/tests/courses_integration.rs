//! PR #12 evidence test — courses storage path end-to-end.
//!
//! Walks: user → product (with manifest in specs_json) → enrollment →
//! mark lesson complete twice (idempotency) → recompute progress_pct →
//! upsert notes without flipping completion → notes survive a re-mark.

use common::ids::OrderId;
use secrecy::SecretString;
use serde_json::json;
use sqlx::PgPool;
use storage::{CourseProgressRepo, EnrollmentsRepo, LicensesRepo, ProductsRepo, UsersRepo};
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
    format!("pr12-{tag}-{}@example.test", Uuid::new_v4())
}

async fn insert_course_product(pool: &PgPool, slug: &str) -> Uuid {
    let id = Uuid::now_v7();
    let manifest = json!({
        "modules": [
            {"id": "m1", "title": "Mod 1", "lessons": [
                {"id": "m1-l1", "title": "Lesson 1"},
                {"id": "m1-l2", "title": "Lesson 2"}
            ]},
            {"id": "m2", "title": "Mod 2", "lessons": [
                {"id": "m2-l1", "title": "Lesson 3"},
                {"id": "m2-l2", "title": "Lesson 4"}
            ]}
        ]
    });
    sqlx::query!(
        r#"
        INSERT INTO products (
            id, slug, kind, name, tagline, description, price_cents,
            highlights, deliverables, requirements, media_poster_color,
            media_accent, specs_json
        ) VALUES (
            $1, $2, 'course', 'PR12 Course', 'tag', 'desc', 19900,
            '{}', '{}', '{}', '#000', '#fff', $3
        )
        "#,
        id,
        slug,
        manifest,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn courses_progress_and_notes_roundtrip() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let products = ProductsRepo::new(pool.clone());
    let licenses = LicensesRepo::new(pool.clone());
    let enrollments = EnrollmentsRepo::new(pool.clone());
    let course_progress = CourseProgressRepo::new(pool.clone());

    let alice = users
        .create(&unique_email("alice"), "Alice", "x")
        .await
        .unwrap();

    let slug = format!("course-pr12-{}", Uuid::new_v4().simple());
    insert_course_product(&pool, &slug).await;
    let product = products.find_by_slug(&slug).await.unwrap().unwrap();

    // Issue enrollment + license via the webhook-equivalent path.
    let pepper = SecretString::from("test-pepper".to_string());
    let order_id = OrderId::new();
    let mut tx = pool.begin().await.unwrap();
    enrollments
        .create_for_purchase_in_tx(&mut tx, alice.id, product.id, order_id)
        .await
        .unwrap();
    licenses
        .issue_for_purchase_in_tx(&mut tx, &pepper, alice.id, product.id, order_id, "CR")
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let enrollment = enrollments
        .find_for_user_and_product(alice.id, product.id)
        .await
        .unwrap()
        .unwrap();
    assert!(enrollment.active);

    // Mark m1-l1 complete (60s). Idempotent re-mark adds the time spent
    // — that's the design (a re-completion implies extra viewing).
    course_progress
        .mark_complete(enrollment.id, "m1-l1", 60, json!({}))
        .await
        .unwrap();
    course_progress
        .mark_complete(enrollment.id, "m1-l1", 30, json!({}))
        .await
        .unwrap();

    let progress = course_progress
        .list_for_enrollment(enrollment.id)
        .await
        .unwrap();
    let m1l1 = progress.iter().find(|p| p.lesson_id == "m1-l1").unwrap();
    assert_eq!(
        m1l1.time_spent_seconds, 90,
        "re-mark must accumulate time_spent"
    );

    // Mark m1-l2 complete. count_completed = 2; total = 4 → 50%.
    course_progress
        .mark_complete(enrollment.id, "m1-l2", 45, json!({}))
        .await
        .unwrap();
    let n = course_progress
        .count_completed(enrollment.id)
        .await
        .unwrap();
    assert_eq!(n, 2);
    enrollments
        .update_progress(enrollment.id, 50, Some("m1-l2"))
        .await
        .unwrap();

    // Notes on a NEW lesson (m2-l1) — should create the row at zero
    // progress (no completion flip).
    course_progress
        .upsert_notes(enrollment.id, "m2-l1", "this is a note")
        .await
        .unwrap();
    let progress = course_progress
        .list_for_enrollment(enrollment.id)
        .await
        .unwrap();
    let m2l1 = progress.iter().find(|p| p.lesson_id == "m2-l1").unwrap();
    assert_eq!(m2l1.notes, "this is a note");
    assert_eq!(
        m2l1.time_spent_seconds, 0,
        "notes-only must not flip completion"
    );
    // count_completed still 2 (m2-l1 has 0s).
    assert_eq!(
        course_progress
            .count_completed(enrollment.id)
            .await
            .unwrap(),
        2
    );

    // Now mark m2-l1 complete — the notes MUST survive.
    course_progress
        .mark_complete(enrollment.id, "m2-l1", 120, json!({}))
        .await
        .unwrap();
    let progress = course_progress
        .list_for_enrollment(enrollment.id)
        .await
        .unwrap();
    let m2l1 = progress.iter().find(|p| p.lesson_id == "m2-l1").unwrap();
    assert_eq!(
        m2l1.notes, "this is a note",
        "mark_complete must NOT clobber existing notes"
    );
    assert_eq!(m2l1.time_spent_seconds, 120);

    // Finish the course. progress_pct = 100 + completed_at stamped.
    course_progress
        .mark_complete(enrollment.id, "m2-l2", 60, json!({}))
        .await
        .unwrap();
    enrollments
        .update_progress(enrollment.id, 100, Some("m2-l2"))
        .await
        .unwrap();

    let row = sqlx::query!(
        "SELECT progress_pct, completed_at FROM enrollments WHERE id = $1",
        enrollment.id.as_uuid()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.progress_pct, 100);
    assert!(
        row.completed_at.is_some(),
        "100% must stamp completed_at exactly once"
    );

    // Re-update to 100% — completed_at MUST NOT move (idempotency).
    let first_completion = row.completed_at.unwrap();
    enrollments
        .update_progress(enrollment.id, 100, Some("m2-l2"))
        .await
        .unwrap();
    let row2 = sqlx::query!(
        "SELECT completed_at FROM enrollments WHERE id = $1",
        enrollment.id.as_uuid()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row2.completed_at.unwrap(),
        first_completion,
        "completed_at must not advance on a re-update to 100%"
    );
}
