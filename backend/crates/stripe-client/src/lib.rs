//! TradeFlex Stripe wrapper.
//!
//! BACKEND.md §1.3 + §8: thin layer over the Stripe REST API. We bring up
//! only the surface PR #7 needs (Customer + Checkout Session creation +
//! webhook signature verification). Subsequent PRs grow the surface.
//!
//! Why not `async-stripe`: see BACKEND_NOTES.md PR #7 decisions. Short
//! version — webhook signature is a 20-line HMAC, REST is straight
//! form-encoded reqwest, and writing the wrapper ourselves keeps the dep
//! tree small and the swap-out cheap (BACKEND.md §23 risk #1).

pub mod api;
pub mod domain;
pub mod recording;
pub mod webhook;

pub use api::{StripeApi, StripeClient, StripeError, StripeInvoice, StripeRefund};
pub use domain::{
    CheckoutLine, CheckoutMode, CheckoutSession, CreateCheckoutArgs, CustomerRef, PortalSession,
};
pub use recording::RecordingStripeApi;
pub use webhook::{verify_signature, ParsedEvent, SignatureError};
