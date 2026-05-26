//! TOTP setup + verify with parameters pinned in code.
//!
//! BACKEND.md §1.2 / §7.3:
//! - Algorithm: SHA-1 (HOTP/TOTP standard; not SHA-256 — Google Authenticator
//!   et al only do SHA-1 reliably).
//! - Digits: 6.
//! - Period: 30 seconds.
//! - **Skew: 1** (current step ± 1, ≈90 s validity).
//!
//! Each value is a literal in the constructor — never relying on a crate
//! default. Skew is the load-bearing setting we want pinned visibly.

use secrecy::ExposeSecret;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::auth::totp_secret_at_rest::SecretBytes;

// Pinned per BACKEND.md §7.3. Centralized so a future change is one line.
const ALGO: Algorithm = Algorithm::SHA1;
pub const DIGITS: usize = 6;
pub const SKEW: u8 = 1;
pub const PERIOD: u64 = 30;

#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    #[error("invalid secret bytes: {0}")]
    Secret(String),
    #[error("invalid totp parameters: {0}")]
    Setup(String),
    #[error("system clock failure")]
    Clock,
}

/// Build a `TOTP` instance bound to `secret`. The `issuer` + `account_name`
/// are baked into the otpauth URI surfaced by `qr_data_url`.
pub fn build(secret: &SecretBytes, issuer: &str, account_name: &str) -> Result<TOTP, TotpError> {
    TOTP::new(
        ALGO,
        DIGITS,
        SKEW,
        PERIOD,
        Secret::Raw(secret.expose_secret().clone())
            .to_bytes()
            .map_err(|e| TotpError::Secret(e.to_string()))?,
        Some(issuer.to_string()),
        account_name.to_string(),
    )
    .map_err(|e| TotpError::Setup(e.to_string()))
}

/// Verify a 6-digit code against the current step ± `SKEW`. Returns
/// `Ok(true)` on match, `Ok(false)` on mismatch.
pub fn verify(totp: &TOTP, code: &str) -> Result<bool, TotpError> {
    if code.len() != DIGITS || !code.chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| TotpError::Clock)?
        .as_secs();
    Ok(totp.check(code, now))
}

/// Verify against a SPECIFIC unix-seconds time. Used by tests to exercise
/// skew boundaries deterministically.
pub fn verify_at(totp: &TOTP, code: &str, at_unix_secs: u64) -> bool {
    if code.len() != DIGITS || !code.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    totp.check(code, at_unix_secs)
}

/// Generate the 6-digit code that would be valid right now. Useful for
/// confirm flow display in dev and for tests; production never returns this
/// to the client.
pub fn current_code(totp: &TOTP) -> Result<String, TotpError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| TotpError::Clock)?
        .as_secs();
    Ok(totp.generate(now))
}

/// Generate the code for a specific unix-seconds time. Test-only.
pub fn code_at(totp: &TOTP, at_unix_secs: u64) -> String {
    totp.generate(at_unix_secs)
}

/// `otpauth://totp/Issuer:account?secret=...&issuer=...&...` — the value the
/// authenticator app scans.
pub fn otpauth_uri(totp: &TOTP) -> String {
    totp.get_url()
}

/// Render the otpauth URI as a base64-encoded PNG data URL. The BFF drops
/// this into `<img src="...">`.
pub fn qr_data_url(totp: &TOTP) -> Result<String, TotpError> {
    totp.get_qr_base64()
        .map(|b64| format!("data:image/png;base64,{b64}"))
        .map_err(|e| TotpError::Setup(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::totp_secret_at_rest::random_secret;

    fn fixed_secret() -> SecretBytes {
        // 20 bytes for SHA-1
        use secrecy::SecretBox;
        SecretBox::new(Box::new(vec![
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
        ]))
    }

    #[test]
    fn build_with_fixed_secret_round_trips_a_code() {
        let totp = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let now = 1_700_000_000_u64; // arbitrary fixed instant
        let code = code_at(&totp, now);
        assert!(verify_at(&totp, &code, now));
    }

    #[test]
    fn skew_one_accepts_prev_and_next_step_only() {
        let totp = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let now = 1_700_000_000_u64;
        let code = code_at(&totp, now);

        // Current step: ok
        assert!(verify_at(&totp, &code, now));
        // Previous step (30 s earlier): SKEW=1 ⇒ accepted
        assert!(verify_at(&totp, &code, now - 30));
        // Next step (30 s later): SKEW=1 ⇒ accepted
        assert!(verify_at(&totp, &code, now + 30));
        // Two steps earlier: rejected
        assert!(!verify_at(&totp, &code, now - 60));
        // Two steps later: rejected
        assert!(!verify_at(&totp, &code, now + 60));
    }

    #[test]
    fn non_digit_or_wrong_length_rejected() {
        let totp = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let now = 1_700_000_000_u64;
        assert!(!verify_at(&totp, "abc123", now));
        assert!(!verify_at(&totp, "12345", now));
        assert!(!verify_at(&totp, "1234567", now));
        assert!(!verify_at(&totp, "", now));
    }

    #[test]
    fn wrong_secret_does_not_validate() {
        let totp_a = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let totp_b = build(&random_secret(), "TradeFlex", "alex@example.com").unwrap();
        let now = 1_700_000_000_u64;
        let code_a = code_at(&totp_a, now);
        assert!(verify_at(&totp_a, &code_a, now));
        assert!(!verify_at(&totp_b, &code_a, now));
    }

    #[test]
    fn otpauth_uri_carries_issuer_and_account() {
        let totp = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let uri = otpauth_uri(&totp);
        // `totp-rs::get_url()` emits only non-default fields; SHA1/6/30 are
        // Google Authenticator defaults and authenticators infer them when
        // absent. We pin the issuer + account + secret presence here; the
        // other params are exercised by the verify tests.
        assert!(uri.starts_with("otpauth://totp/"), "got {uri}");
        assert!(uri.contains("issuer=TradeFlex"), "got {uri}");
        assert!(uri.contains("alex"), "got {uri}");
        assert!(uri.contains("secret="), "got {uri}");
    }

    #[test]
    fn qr_data_url_is_base64_png() {
        let totp = build(&fixed_secret(), "TradeFlex", "alex@example.com").unwrap();
        let url = qr_data_url(&totp).unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
        // Strip prefix and decode a few bytes — non-empty.
        let b64 = url.trim_start_matches("data:image/png;base64,");
        assert!(b64.len() > 100);
    }
}
