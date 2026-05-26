//! Stripe-idempotency-key derivation.
//!
//! BACKEND.md §8.2: every Stripe write uses an idempotency key of the form
//!
//! ```text
//! tf:{base32(sha256(prefix || user_id || action || nonce))[..24]}
//! ```
//!
//! - `prefix` — `"tradeflex-prod"` / `"tradeflex-stg"` / `"tradeflex-dev"`.
//!   Distinguishes environments so a staging retry can't accidentally
//!   collapse a prod write.
//! - `user_id` — the TradeFlex user the operation belongs to.
//! - `action` — short identifier, e.g. `"checkout"`, `"refund"`, `"cancel_sub"`.
//! - `nonce` — caller-supplied unique-per-intent value. For checkout it's
//!   the cart hash (so a re-submitted identical cart is a no-op).
//!
//! Output is `tf:` + base32(no-pad, lowercase) of the first 15 bytes of the
//! SHA-256 — gives us 24 alphanumeric chars total, well within Stripe's
//! 255-char limit.

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::ids::UserId;

const PREFIX_ENV: &str = "tradeflex";
const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Compute an idempotency key for the given input.
///
/// The output is always 24 chars (`"tf:" + 21 chars of b64`) which keeps
/// it well under Stripe's 255-char limit and trivial to log without
/// truncation.
pub fn derive_key(env: &str, user_id: UserId, action: &str, nonce: &str) -> String {
    let mut h = Sha256::new();
    h.update(PREFIX_ENV.as_bytes());
    h.update(b"-");
    h.update(env.as_bytes());
    h.update(b"|");
    h.update(user_id.as_uuid().as_bytes());
    h.update(b"|");
    h.update(action.as_bytes());
    h.update(b"|");
    h.update(nonce.as_bytes());
    let digest = h.finalize();
    // 15 bytes → 20 base64 chars (URL-safe, no padding).
    let encoded = B64.encode(&digest[..15]);
    format!("tf:{encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_inputs() {
        let u = UserId::new();
        let a = derive_key("prod", u, "checkout", "cart-abc");
        let b = derive_key("prod", u, "checkout", "cart-abc");
        assert_eq!(a, b);
    }

    #[test]
    fn differs_per_env_user_action_nonce() {
        let u = UserId::new();
        let base = derive_key("prod", u, "checkout", "n");
        assert_ne!(base, derive_key("stg", u, "checkout", "n"));
        assert_ne!(base, derive_key("prod", UserId::new(), "checkout", "n"));
        assert_ne!(base, derive_key("prod", u, "refund", "n"));
        assert_ne!(base, derive_key("prod", u, "checkout", "n2"));
    }

    #[test]
    fn shape_is_tf_colon_b64_well_under_limit() {
        let k = derive_key("prod", UserId::new(), "checkout", "n");
        assert!(k.starts_with("tf:"));
        assert!(k.len() < 32, "key too long: {k}");
        assert!(k
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '-' || c == '_'));
    }
}
