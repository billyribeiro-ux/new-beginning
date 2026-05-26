//! Domain types the wrapper exposes. Stripe-specific shapes stay private to
//! `api.rs`; everything that crosses the crate boundary is our own type.

use common::ids::{OrderId, UserId};
use serde::{Deserialize, Serialize};

/// Stripe customer reference. We never inline the full `Customer` resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRef {
    pub id: String, // e.g. "cus_..."
}

/// What kind of Checkout Session to mint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckoutMode {
    /// One-time purchase of one or more products. Matches Stripe's
    /// `mode=payment`.
    Payment,
    /// New subscription. Matches Stripe's `mode=subscription`.
    Subscription,
}

impl CheckoutMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CheckoutMode::Payment => "payment",
            CheckoutMode::Subscription => "subscription",
        }
    }
}

/// Each line item the Checkout Session shows. `price_id` is the Stripe
/// `price_...` id; `quantity` is positive.
#[derive(Debug, Clone)]
pub struct CheckoutLine {
    pub price_id: String,
    pub quantity: i64,
}

/// Inputs for `create_checkout_session`. Owned types so the caller can build
/// the args without lifetime threading.
#[derive(Debug, Clone)]
pub struct CreateCheckoutArgs {
    pub mode: CheckoutMode,
    pub customer: CustomerRef,
    /// The TradeFlex order row; surfaced to Stripe via `client_reference_id`
    /// and `metadata.order_id` so the webhook handler can find the order.
    pub order_id: OrderId,
    pub user_id: UserId,
    pub lines: Vec<CheckoutLine>,
    pub success_url: String,
    pub cancel_url: String,
}

/// Checkout Session created by Stripe. We surface only the fields the BFF
/// + webhook handler care about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: String,
}

/// Customer Portal session — Stripe-hosted billing self-serve.
/// Short-lived URL; the BFF redirects the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSession {
    pub id: String,
    pub url: String,
}
