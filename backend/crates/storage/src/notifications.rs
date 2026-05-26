//! `NotificationsRepo` + `NotificationPreferencesRepo`.
//!
//! BACKEND.md §4 0017 + §11 (notifications surface).
//!
//! Two tables:
//!
//! * `notifications` — per-user in-app feed. One row per delivered
//!   notification. `read_at` flips when the user marks it read.
//! * `notification_preferences` — per-user opt-in matrix across kinds
//!   (product_updates / billing / course_progress / market_alerts /
//!   marketing) × channels (email / inapp), plus DND window.
//!
//! The `dispatch_notification` job (PR #13 worker hook) takes an
//! "intent" (kind + actor + payload) and fans it out per preference: it
//! inserts an in-app row if `*_inapp = TRUE`, and enqueues a mail job
//! if `*_email = TRUE`. DND defers the in-app row's `created_at` (out of
//! scope for the storage repo — handler-side filter only).

use common::ids::{NotificationId, UserId};
use serde_json::Value as Json;
use sqlx::PgPool;
use time::{OffsetDateTime, Time};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NotificationPreferences {
    pub user_id: UserId,
    pub product_updates_email: bool,
    pub product_updates_inapp: bool,
    pub billing_email: bool,
    pub billing_inapp: bool,
    pub course_progress_email: bool,
    pub course_progress_inapp: bool,
    pub market_alerts_email: bool,
    pub market_alerts_inapp: bool,
    pub marketing_email: bool,
    pub marketing_inapp: bool,
    pub dnd_enabled: bool,
    pub dnd_start: Option<Time>,
    pub dnd_end: Option<Time>,
    pub timezone: String,
    pub updated_at: OffsetDateTime,
}

/// Mapping a notification `kind` slug to (email_opt_in, inapp_opt_in)
/// from the preferences row. Centralized so adding a new kind is one
/// place to update.
pub fn channels_for_kind(prefs: &NotificationPreferences, kind: &str) -> (bool, bool) {
    match kind {
        k if k.starts_with("product.") => {
            (prefs.product_updates_email, prefs.product_updates_inapp)
        }
        k if k.starts_with("billing.")
            || k.starts_with("invoice.")
            || k.starts_with("subscription.") =>
        {
            (prefs.billing_email, prefs.billing_inapp)
        }
        k if k.starts_with("course.") => (prefs.course_progress_email, prefs.course_progress_inapp),
        k if k.starts_with("market.") => (prefs.market_alerts_email, prefs.market_alerts_inapp),
        k if k.starts_with("marketing.") => (prefs.marketing_email, prefs.marketing_inapp),
        // Unknown / system kinds always deliver in-app, never email. The
        // closed set above is the only one we'll surface to UI prefs.
        _ => (false, true),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationsError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct NotificationsRepo {
    pool: PgPool,
}

impl NotificationsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a notification row. `payload` is opaque to the repo; the
    /// dispatch job decides shape. Called from both the api binary
    /// (e.g. a course-completion in-flow notification) and the worker
    /// (e.g. an admin broadcast).
    pub async fn create(
        &self,
        user_id: UserId,
        kind: &str,
        title: &str,
        body: &str,
        source: &str,
    ) -> Result<NotificationId, NotificationsError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO notifications (id, user_id, kind, title, body, source)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            id,
            user_id.as_uuid(),
            kind,
            title,
            body,
            source,
        )
        .execute(&self.pool)
        .await?;
        Ok(NotificationId::from_uuid(id))
    }

    /// Feed for the user. `limit` caps; `unread_only` skips read rows.
    pub async fn list_for_user(
        &self,
        user_id: UserId,
        unread_only: bool,
        limit: i64,
    ) -> Result<Vec<Notification>, NotificationsError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, kind, title, body, source, read_at, created_at
            FROM notifications
            WHERE user_id = $1
              AND ($2 = FALSE OR read_at IS NULL)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            user_id.as_uuid(),
            unread_only,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Notification {
                id: NotificationId::from_uuid(r.id),
                user_id: UserId::from_uuid(r.user_id),
                kind: r.kind,
                title: r.title,
                body: r.body,
                source: r.source,
                read_at: r.read_at,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn mark_read(
        &self,
        user_id: UserId,
        id: NotificationId,
    ) -> Result<u64, NotificationsError> {
        let res = sqlx::query!(
            r#"
            UPDATE notifications
            SET read_at = now()
            WHERE id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
            id.as_uuid(),
            user_id.as_uuid(),
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn mark_all_read(&self, user_id: UserId) -> Result<u64, NotificationsError> {
        let res = sqlx::query!(
            r#"
            UPDATE notifications
            SET read_at = now()
            WHERE user_id = $1 AND read_at IS NULL
            "#,
            user_id.as_uuid(),
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn unread_count(&self, user_id: UserId) -> Result<i64, NotificationsError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM notifications
            WHERE user_id = $1 AND read_at IS NULL
            "#,
            user_id.as_uuid(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count)
    }
}

#[derive(Clone)]
pub struct NotificationPreferencesRepo {
    pool: PgPool,
}

#[derive(Debug, Clone, Default)]
pub struct PreferencesPatch {
    pub product_updates_email: Option<bool>,
    pub product_updates_inapp: Option<bool>,
    pub billing_email: Option<bool>,
    pub billing_inapp: Option<bool>,
    pub course_progress_email: Option<bool>,
    pub course_progress_inapp: Option<bool>,
    pub market_alerts_email: Option<bool>,
    pub market_alerts_inapp: Option<bool>,
    pub marketing_email: Option<bool>,
    pub marketing_inapp: Option<bool>,
    pub dnd_enabled: Option<bool>,
    pub dnd_start: Option<Option<Time>>,
    pub dnd_end: Option<Option<Time>>,
    pub timezone: Option<String>,
}

impl NotificationPreferencesRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fetch the preferences row, inserting defaults if absent. This is
    /// the lazy materialization path — we don't write the row at user
    /// signup, only on first read or first PATCH.
    pub async fn get_or_default(
        &self,
        user_id: UserId,
    ) -> Result<NotificationPreferences, NotificationsError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO notification_preferences (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING
            RETURNING user_id, product_updates_email, product_updates_inapp,
                      billing_email, billing_inapp,
                      course_progress_email, course_progress_inapp,
                      market_alerts_email, market_alerts_inapp,
                      marketing_email, marketing_inapp,
                      dnd_enabled, dnd_start, dnd_end, timezone, updated_at
            "#,
            user_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await?;
        if let Some(r) = row {
            return Ok(NotificationPreferences {
                user_id: UserId::from_uuid(r.user_id),
                product_updates_email: r.product_updates_email,
                product_updates_inapp: r.product_updates_inapp,
                billing_email: r.billing_email,
                billing_inapp: r.billing_inapp,
                course_progress_email: r.course_progress_email,
                course_progress_inapp: r.course_progress_inapp,
                market_alerts_email: r.market_alerts_email,
                market_alerts_inapp: r.market_alerts_inapp,
                marketing_email: r.marketing_email,
                marketing_inapp: r.marketing_inapp,
                dnd_enabled: r.dnd_enabled,
                dnd_start: r.dnd_start,
                dnd_end: r.dnd_end,
                timezone: r.timezone,
                updated_at: r.updated_at,
            });
        }
        // Row already existed; fetch.
        let r = sqlx::query!(
            r#"
            SELECT user_id, product_updates_email, product_updates_inapp,
                   billing_email, billing_inapp,
                   course_progress_email, course_progress_inapp,
                   market_alerts_email, market_alerts_inapp,
                   marketing_email, marketing_inapp,
                   dnd_enabled, dnd_start, dnd_end, timezone, updated_at
            FROM notification_preferences
            WHERE user_id = $1
            "#,
            user_id.as_uuid(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(NotificationPreferences {
            user_id: UserId::from_uuid(r.user_id),
            product_updates_email: r.product_updates_email,
            product_updates_inapp: r.product_updates_inapp,
            billing_email: r.billing_email,
            billing_inapp: r.billing_inapp,
            course_progress_email: r.course_progress_email,
            course_progress_inapp: r.course_progress_inapp,
            market_alerts_email: r.market_alerts_email,
            market_alerts_inapp: r.market_alerts_inapp,
            marketing_email: r.marketing_email,
            marketing_inapp: r.marketing_inapp,
            dnd_enabled: r.dnd_enabled,
            dnd_start: r.dnd_start,
            dnd_end: r.dnd_end,
            timezone: r.timezone,
            updated_at: r.updated_at,
        })
    }

    /// Patch preferences. Unset fields stay unchanged. Always materializes
    /// the row first (so a PATCH against a user who's never visited
    /// /preferences still works).
    pub async fn patch(
        &self,
        user_id: UserId,
        p: PreferencesPatch,
    ) -> Result<NotificationPreferences, NotificationsError> {
        self.get_or_default(user_id).await?;
        sqlx::query!(
            r#"
            UPDATE notification_preferences
            SET product_updates_email = COALESCE($2, product_updates_email),
                product_updates_inapp = COALESCE($3, product_updates_inapp),
                billing_email = COALESCE($4, billing_email),
                billing_inapp = COALESCE($5, billing_inapp),
                course_progress_email = COALESCE($6, course_progress_email),
                course_progress_inapp = COALESCE($7, course_progress_inapp),
                market_alerts_email = COALESCE($8, market_alerts_email),
                market_alerts_inapp = COALESCE($9, market_alerts_inapp),
                marketing_email = COALESCE($10, marketing_email),
                marketing_inapp = COALESCE($11, marketing_inapp),
                dnd_enabled = COALESCE($12, dnd_enabled),
                dnd_start = CASE WHEN $13::bool THEN $14::time ELSE dnd_start END,
                dnd_end = CASE WHEN $15::bool THEN $16::time ELSE dnd_end END,
                timezone = COALESCE($17, timezone),
                updated_at = now()
            WHERE user_id = $1
            "#,
            user_id.as_uuid(),
            p.product_updates_email,
            p.product_updates_inapp,
            p.billing_email,
            p.billing_inapp,
            p.course_progress_email,
            p.course_progress_inapp,
            p.market_alerts_email,
            p.market_alerts_inapp,
            p.marketing_email,
            p.marketing_inapp,
            p.dnd_enabled,
            // double-Option semantics: outer Some = "user sent this
            // field"; inner Some = "set to this value", inner None =
            // "set to NULL". We encode "user sent it" as a bool flag.
            p.dnd_start.is_some(),
            p.dnd_start.unwrap_or(None),
            p.dnd_end.is_some(),
            p.dnd_end.unwrap_or(None),
            p.timezone,
        )
        .execute(&self.pool)
        .await?;
        self.get_or_default(user_id).await
    }
}

/// Unused for now; the `_payload` is reserved for the dispatch worker
/// (PR #13 wires it). Keeping the type lets a future change avoid
/// hitting call sites.
pub type NotificationPayload = Json;
