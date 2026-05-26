//! Re-driveable jobs that the worker (PR #11) will schedule.
//!
//! PR #8 lands the bodies as plain async fns; PR #11 wires them into a
//! cron loop. Living in the `http` crate is a temporary home — when the
//! `jobs` crate lands (BACKEND.md §2), the fns move there unchanged.

pub mod fetch_invoice_pdfs;
pub mod reconcile_stripe_events;
