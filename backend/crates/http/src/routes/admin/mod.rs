//! `/v1/admin/**` — admin-only handlers.
//!
//! BACKEND.md §11 (endpoint surface), §22 rule 8 (audit + mutate in tx).
//!
//! Every route under this tree is gated by `require_admin` in `app.rs`;
//! the gating is centralized so a handler never needs to re-check the
//! role itself. Every mutating handler MUST `audit::record_in_tx` in the
//! same transaction as the mutation.

pub mod customers;
pub mod kpis;
pub mod leads;
pub mod messages;
pub mod products;
pub mod refund;
