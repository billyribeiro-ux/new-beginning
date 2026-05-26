//! `POST /v1/billing/portal` — Stripe Customer Portal hand-off.
//!
//! BACKEND.md §8.6. Returns `{url}`; the BFF redirects the browser. The
//! portal is where the user updates payment methods, views invoices,
//! cancels (at-period-end), and switches plans. Changes ride back via
//! the same webhook stack PR #9 wires up.

use axum::extract::State;
use axum::Json;
use common::auth::idempotency;
use common::error::AppError;
use serde::Serialize;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct PortalResponse {
    pub url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<PortalResponse>, AppError> {
    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;

    // Resolve the customer ref. Two paths:
    //   1. Already-known `users.stripe_customer_id`.
    //   2. Create on-the-fly (the user hasn't checked out yet but wants
    //      to land on the portal — e.g. to add a payment method ahead of
    //      time). Idempotency-key on user_id.
    let customer = if let Some(cid) = user.stripe_customer_id.clone() {
        stripe_client::CustomerRef { id: cid }
    } else {
        let idem = idempotency::derive_key(
            &state.config.env,
            session.user_id,
            "portal_customer",
            "first",
        );
        let cust = state
            .stripe
            .get_or_create_customer_for_user(
                session.user_id,
                &user.email,
                Some(&user.name),
                Some(&idem),
            )
            .await
            .map_err(|e| AppError::External {
                service: "stripe",
                source: anyhow::anyhow!(e),
            })?;
        state
            .users
            .set_stripe_customer_id(session.user_id, &cust.id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        cust
    };

    let return_url = format!(
        "{}/dashboard/billing",
        state.config.public_site_url.trim_end_matches('/')
    );
    let idem = idempotency::derive_key(
        &state.config.env,
        session.user_id,
        "portal_session",
        // Short-TTL → nonce includes the minute bucket so concurrent
        // taps collapse but the next tap a minute later gets a fresh
        // session.
        &(time::OffsetDateTime::now_utc().unix_timestamp() / 60).to_string(),
    );
    let session_url = state
        .stripe
        .create_customer_portal_session(&customer, &return_url, Some(&idem))
        .await
        .map_err(|e| AppError::External {
            service: "stripe",
            source: anyhow::anyhow!(e),
        })?;
    Ok(Json(PortalResponse {
        url: session_url.url,
    }))
}
