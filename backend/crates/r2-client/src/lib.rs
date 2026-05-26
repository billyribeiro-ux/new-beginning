//! TradeFlex R2 / S3 wrapper.
//!
//! BACKEND.md §1.4 + §9. Cloudflare R2's S3-compatible API takes SigV4 PUT
//! / GET requests and pre-signed URLs the same way AWS S3 does, so we use
//! `rusty-s3` (purpose-built for SigV4 presigning, ~50 deps lighter than
//! `aws-sdk-s3`). The trait abstraction means tests + evidence runs use a
//! `RecordingObjectStore` fake that captures every call.

pub mod api;
pub mod keys;
pub mod recording;

pub use api::{ObjectStore, R2Client, StoreError};
pub use recording::RecordingObjectStore;
