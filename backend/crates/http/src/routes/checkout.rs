//! `POST /v1/checkout`.
//!
//! BACKEND.md §8.2 step-by-step:
//! 1. Validate cart against current DB prices (NEVER trust client cents).
//! 2. Create a `pending` `orders` row + its `order_items`.
//! 3. Get-or-create the user's Stripe customer.
//! 4. Create the Stripe Checkout Session with `client_reference_id = order_id`.
//! 5. Attach the session id to the order so the webhook handler can resolve.
//! 6. Return `{url}` for the BFF to redirect to.
//!
//! Cart shape mirrors `src/lib/stores/cart.svelte.ts::CartLine`. A mixed
//! cart (subscription + product) is rejected because Stripe Checkout's
//! `mode=subscription` is single-recurring-only.

use axum::extract::State;
use axum::Json;
use common::auth::idempotency;
use common::error::AppError;
use common::ids::{PlanId, ProductId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage::{CartSnapshotLine, NewOrderItem};
use stripe_client::{CheckoutLine, CheckoutMode, CreateCheckoutArgs};
use time::Duration;
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

const CHECKOUT_PENDING_TTL: Duration = Duration::hours(1);

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CheckoutRequest {
    #[validate(length(min = 1, max = 20))]
    pub lines: Vec<CheckoutLineInput>,
    /// Optional override; defaults to `{public_site_url}/checkout/success`.
    pub success_url: Option<String>,
    /// Optional override; defaults to `{public_site_url}/cart`.
    pub cancel_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CheckoutLineInput {
    /// `"product"` or `"plan"`.
    pub kind: String,
    /// Slug; we look up the row + use its current price. Client-supplied
    /// price is intentionally ignored.
    #[validate(length(min = 1, max = 128))]
    pub slug: String,
    #[validate(range(min = 1, max = 99))]
    pub quantity: i64,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub order_id: String,
}

pub async fn handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("invalid checkout payload".into()))?;
    for l in &body.lines {
        l.validate()
            .map_err(|_| AppError::Validation("invalid checkout line".into()))?;
    }

    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;

    let mut items: Vec<NewOrderItem> = Vec::with_capacity(body.lines.len());
    let mut snapshot: Vec<CartSnapshotLine> = Vec::with_capacity(body.lines.len());
    let mut checkout_lines: Vec<CheckoutLine> = Vec::with_capacity(body.lines.len());
    let mut mode: Option<CheckoutMode> = None;
    let mut subtotal: i64 = 0;

    for line in &body.lines {
        match line.kind.as_str() {
            "product" => {
                if matches!(mode, Some(CheckoutMode::Subscription)) {
                    return Err(AppError::Validation(
                        "cart cannot mix subscription with one-time products".into(),
                    ));
                }
                mode = Some(CheckoutMode::Payment);
                let p = state
                    .products
                    .find_by_slug(&line.slug)
                    .await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
                    .ok_or_else(|| {
                        AppError::Validation(format!("unknown product slug: {}", line.slug))
                    })?;
                let line_total = p
                    .price_cents
                    .checked_mul(line.quantity)
                    .ok_or_else(|| AppError::Validation("line total overflow".into()))?;
                subtotal = subtotal
                    .checked_add(line_total)
                    .ok_or_else(|| AppError::Validation("subtotal overflow".into()))?;
                let price_id = p.id.to_string(); // PR #7 placeholder
                items.push(NewOrderItem {
                    product_id: Some(ProductId::from_uuid(p.id.as_uuid())),
                    plan_id: None,
                    quantity: line.quantity as i32,
                    unit_price_cents: p.price_cents,
                    line_total_cents: line_total,
                    name_snapshot: p.name.clone(),
                    slug_snapshot: p.slug.clone(),
                });
                snapshot.push(CartSnapshotLine {
                    kind: "product".into(),
                    slug: p.slug.clone(),
                    name: p.name.clone(),
                    price_cents: p.price_cents,
                    quantity: line.quantity,
                });
                let stripe_price_id = product_stripe_price_id(&p, &price_id)?;
                checkout_lines.push(CheckoutLine {
                    price_id: stripe_price_id,
                    quantity: line.quantity,
                });
            }
            "plan" => {
                if matches!(mode, Some(CheckoutMode::Payment)) {
                    return Err(AppError::Validation(
                        "cart cannot mix subscription with one-time products".into(),
                    ));
                }
                if body.lines.len() > 1 {
                    return Err(AppError::Validation(
                        "subscription checkout must be a single plan".into(),
                    ));
                }
                mode = Some(CheckoutMode::Subscription);
                let p = state
                    .plans
                    .find_by_slug(&line.slug)
                    .await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
                    .ok_or_else(|| {
                        AppError::Validation(format!("unknown plan slug: {}", line.slug))
                    })?;
                let line_total = p
                    .price_cents
                    .checked_mul(line.quantity)
                    .ok_or_else(|| AppError::Validation("line total overflow".into()))?;
                subtotal = subtotal
                    .checked_add(line_total)
                    .ok_or_else(|| AppError::Validation("subtotal overflow".into()))?;
                items.push(NewOrderItem {
                    product_id: None,
                    plan_id: Some(PlanId::from_uuid(p.id.as_uuid())),
                    quantity: line.quantity as i32,
                    unit_price_cents: p.price_cents,
                    line_total_cents: line_total,
                    name_snapshot: p.name.clone(),
                    slug_snapshot: p.slug.clone(),
                });
                snapshot.push(CartSnapshotLine {
                    kind: "plan".into(),
                    slug: p.slug.clone(),
                    name: p.name.clone(),
                    price_cents: p.price_cents,
                    quantity: line.quantity,
                });
                checkout_lines.push(CheckoutLine {
                    price_id: p.stripe_price_id.clone(),
                    quantity: line.quantity,
                });
            }
            other => {
                return Err(AppError::Validation(format!(
                    "unknown cart line kind: {other}"
                )));
            }
        }
    }
    let mode = mode.ok_or_else(|| AppError::Validation("cart is empty".into()))?;

    // Persist the pending order BEFORE we call Stripe — if Stripe errors,
    // the order row stays around (and gets expired by the worker cron).
    let order = state
        .orders
        .create_pending(
            session.user_id,
            "usd",
            subtotal,
            0,
            &items,
            &snapshot,
            CHECKOUT_PENDING_TTL,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // Mint the Stripe customer + checkout session under idempotency keys
    // derived from a deterministic cart hash, so a retried checkout for the
    // same cart collapses to one Stripe write.
    let cart_hash = hash_cart(&body.lines);
    let customer_idem = idempotency::derive_key(
        &state.config.env,
        session.user_id,
        "checkout_customer",
        &cart_hash,
    );
    let customer = state
        .stripe
        .get_or_create_customer_for_user(
            session.user_id,
            &user.email,
            Some(&user.name),
            Some(&customer_idem),
        )
        .await
        .map_err(|e| AppError::External {
            service: "stripe",
            source: anyhow::anyhow!(e),
        })?;

    let session_idem = idempotency::derive_key(
        &state.config.env,
        session.user_id,
        "checkout_session",
        &format!("{}-{cart_hash}", order.id),
    );
    let public = state.config.public_site_url.trim_end_matches('/');
    let success_url = body
        .success_url
        .unwrap_or_else(|| format!("{public}/checkout/success?order={}", order.id));
    let cancel_url = body.cancel_url.unwrap_or_else(|| format!("{public}/cart"));

    let checkout = state
        .stripe
        .create_checkout_session(
            CreateCheckoutArgs {
                mode,
                customer,
                order_id: order.id,
                user_id: session.user_id,
                lines: checkout_lines,
                success_url,
                cancel_url,
            },
            Some(&session_idem),
        )
        .await
        .map_err(|e| AppError::External {
            service: "stripe",
            source: anyhow::anyhow!(e),
        })?;

    state
        .orders
        .attach_stripe_checkout_session(order.id, &checkout.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(CheckoutResponse {
        checkout_url: checkout.url,
        order_id: order.id.to_string(),
    }))
}

/// BACKEND.md §8.2: the seeder writes a deterministic placeholder
/// `stripe_price_id` (`price_test_TF_{slug}`); PR #14's admin product CRUD
/// overwrites with a real Stripe Price id. Missing → 400 (we never
/// guess).
fn product_stripe_price_id(p: &storage::Product, _fallback: &str) -> Result<String, AppError> {
    p.stripe_price_id.clone().ok_or_else(|| {
        AppError::Validation(format!(
            "product '{}' has no stripe_price_id — operator must wire one",
            p.slug
        ))
    })
}

fn hash_cart(lines: &[CheckoutLineInput]) -> String {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let mut h = Sha256::new();
    for l in lines {
        h.update(l.kind.as_bytes());
        h.update(b"|");
        h.update(l.slug.as_bytes());
        h.update(b"|");
        h.update(l.quantity.to_be_bytes());
        h.update(b"\n");
    }
    let digest = h.finalize();
    b64.encode(&digest[..12])
}
