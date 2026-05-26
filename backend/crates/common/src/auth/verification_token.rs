//! Single-use verification tokens (email-change, password-reset, signup-verify).
//!
//! Unlike session cookies, these go in an email-delivered link and have no
//! HMAC — the token itself is the secret. The DB stores `sha256(token)`;
//! plaintext lives only in the link until the user clicks.
//!
//! The plaintext is base64-url with no padding so it's URL-safe and easy to
//! drop into a query string.

use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use secrecy::SecretString;
use sha2::{Digest, Sha256};

const TOKEN_BYTES: usize = 32;
const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// A freshly minted token. Hand `plaintext` to the mailer (or, until PR-X,
/// log it); persist `hash`.
pub struct IssuedToken {
    pub plaintext: SecretString,
    pub hash: [u8; 32],
}

pub fn issue() -> IssuedToken {
    let mut bytes = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let plaintext = SecretString::from(B64.encode(bytes));
    let mut h = Sha256::new();
    h.update(bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&h.finalize());
    IssuedToken { plaintext, hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn issued_tokens_are_unique() {
        let a = issue();
        let b = issue();
        assert_ne!(a.plaintext.expose_secret(), b.plaintext.expose_secret());
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn plaintext_is_url_safe_base64_no_padding() {
        let t = issue();
        let s = t.plaintext.expose_secret();
        assert!(!s.contains('='), "should be no-pad: {s}");
        assert!(s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        // 32 bytes → ceil(32 * 4 / 3) = 43 chars (no padding).
        assert_eq!(s.len(), 43);
    }
}
