//! Stage-1 → stage-2 hand-off token for two-step login.
//!
//! BACKEND.md §19. When `POST /v1/auth/login` succeeds but the user has 2FA
//! enabled, we don't set a session cookie yet — we return a short-lived
//! `tx_id` that authorizes a follow-up `POST /v1/auth/login/totp`.
//!
//! Wire format (mirrors `session_cookie.rs` so the verification discipline
//! is the same; constant-time HMAC, version-prefixed):
//!
//! ```text
//! v1.<b64url(user_id)>.<b64url(expires_unix_secs_be)>.<b64url(hmac_sha256(key, "tfx_pending_totp." + version + "." + uid + "." + exp))>
//! ```
//!
//! The domain separator `tfx_pending_totp.` makes the MAC reject any cookie
//! signed for a different surface even if the same key is reused.

use base64::Engine;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::ids::UserId;

const VERSION: &str = "v1";
const DOMAIN: &str = "tfx_pending_totp";
const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum PendingTotpError {
    #[error("malformed pending_totp token")]
    Malformed,
    #[error("invalid pending_totp signature")]
    BadSignature,
    #[error("unsupported pending_totp version")]
    UnknownVersion,
    #[error("pending_totp token expired")]
    Expired,
    #[error("hmac key invalid: {0}")]
    KeySetup(String),
    #[error("system clock failure")]
    Clock,
}

pub fn issue(
    user_id: UserId,
    ttl: std::time::Duration,
    key: &SecretString,
) -> Result<String, PendingTotpError> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PendingTotpError::Clock)?
        .as_secs()
        + ttl.as_secs();
    encode(user_id, exp, key)
}

pub fn verify(raw: &str, key: &SecretString) -> Result<UserId, PendingTotpError> {
    let mut parts = raw.split('.');
    let version = parts.next().ok_or(PendingTotpError::Malformed)?;
    let uid_b64 = parts.next().ok_or(PendingTotpError::Malformed)?;
    let exp_b64 = parts.next().ok_or(PendingTotpError::Malformed)?;
    let sig_b64 = parts.next().ok_or(PendingTotpError::Malformed)?;
    if parts.next().is_some() {
        return Err(PendingTotpError::Malformed);
    }
    if version != VERSION {
        return Err(PendingTotpError::UnknownVersion);
    }

    let uid_bytes = B64
        .decode(uid_b64)
        .map_err(|_| PendingTotpError::Malformed)?;
    let exp_bytes = B64
        .decode(exp_b64)
        .map_err(|_| PendingTotpError::Malformed)?;
    let sig_bytes = B64
        .decode(sig_b64)
        .map_err(|_| PendingTotpError::Malformed)?;
    if uid_bytes.len() != 16 || exp_bytes.len() != 8 {
        return Err(PendingTotpError::Malformed);
    }

    let expected = mac(key, version, uid_b64, exp_b64)?;
    if !bool::from(expected.ct_eq(&sig_bytes)) {
        return Err(PendingTotpError::BadSignature);
    }

    let exp_unix = u64::from_be_bytes(exp_bytes.try_into().unwrap());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PendingTotpError::Clock)?
        .as_secs();
    if exp_unix <= now {
        return Err(PendingTotpError::Expired);
    }

    let mut uuid_buf = [0u8; 16];
    uuid_buf.copy_from_slice(&uid_bytes);
    Ok(UserId::from_uuid(Uuid::from_bytes(uuid_buf)))
}

fn encode(user_id: UserId, exp_unix: u64, key: &SecretString) -> Result<String, PendingTotpError> {
    let uid_b64 = B64.encode(user_id.as_uuid().as_bytes());
    let exp_b64 = B64.encode(exp_unix.to_be_bytes());
    let m = mac(key, VERSION, &uid_b64, &exp_b64)?;
    let sig_b64 = B64.encode(m);
    Ok(format!("{VERSION}.{uid_b64}.{exp_b64}.{sig_b64}"))
}

fn mac(
    key: &SecretString,
    version: &str,
    uid_b64: &str,
    exp_b64: &str,
) -> Result<Vec<u8>, PendingTotpError> {
    let key_bytes = key.expose_secret().as_bytes();
    let mut h = HmacSha256::new_from_slice(key_bytes)
        .map_err(|e| PendingTotpError::KeySetup(e.to_string()))?;
    // Domain separator first so a MAC from a different surface (session
    // cookie) cannot be replayed here even with the same key.
    h.update(DOMAIN.as_bytes());
    h.update(b".");
    h.update(version.as_bytes());
    h.update(b".");
    h.update(uid_b64.as_bytes());
    h.update(b".");
    h.update(exp_b64.as_bytes());
    Ok(h.finalize().into_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn key() -> SecretString {
        SecretString::from("a-test-key-of-sufficient-length-for-hmac".to_string())
    }

    #[test]
    fn issue_then_verify_round_trip() {
        let uid = UserId::new();
        let tok = issue(uid, Duration::from_secs(300), &key()).unwrap();
        let recovered = verify(&tok, &key()).unwrap();
        assert_eq!(recovered, uid);
    }

    #[test]
    fn rejects_tampered_user_id() {
        let tok = issue(UserId::new(), Duration::from_secs(300), &key()).unwrap();
        let parts: Vec<&str> = tok.split('.').collect();
        let mut new_uid: Vec<char> = parts[1].chars().collect();
        // Flip a char in the MIDDLE of the segment. The last char of a
        // 22-char b64 string for a 16-byte UUID has 4 padding bits, so
        // some flips there decode to identical bytes — false negative.
        // The 5th char encodes only useful bits and is safe to tamper.
        let i = 4.min(new_uid.len().saturating_sub(1));
        new_uid[i] = if new_uid[i] == 'A' { 'B' } else { 'A' };
        let tampered_uid: String = new_uid.into_iter().collect();
        let tampered = format!("{}.{}.{}.{}", parts[0], tampered_uid, parts[2], parts[3]);
        let err = verify(&tampered, &key()).unwrap_err();
        assert!(
            matches!(
                err,
                PendingTotpError::BadSignature | PendingTotpError::Malformed
            ),
            "unexpected: {err:?}"
        );
    }

    #[test]
    fn rejects_wrong_key() {
        let tok = issue(UserId::new(), Duration::from_secs(300), &key()).unwrap();
        let other = SecretString::from("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string());
        let err = verify(&tok, &other).unwrap_err();
        assert!(matches!(err, PendingTotpError::BadSignature));
    }

    #[test]
    fn rejects_expired_token() {
        // Encode an explicit past expiry.
        let uid = UserId::new();
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 1;
        let tok = encode(uid, exp, &key()).unwrap();
        let err = verify(&tok, &key()).unwrap_err();
        assert!(matches!(err, PendingTotpError::Expired));
    }

    #[test]
    fn rejects_session_cookie_shaped_input() {
        // A session-cookie MAC built with the same key would NOT validate
        // here because the domain separator `tfx_pending_totp` is inside
        // the HMAC payload. Hand-craft a near-shape and confirm.
        let uid = UserId::new();
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 300;
        let uid_b64 = B64.encode(uid.as_uuid().as_bytes());
        let exp_b64 = B64.encode(exp.to_be_bytes());
        // Build a MAC WITHOUT the domain separator — i.e. wrong.
        let mut h = HmacSha256::new_from_slice(key().expose_secret().as_bytes()).unwrap();
        h.update(VERSION.as_bytes());
        h.update(b".");
        h.update(uid_b64.as_bytes());
        h.update(b".");
        h.update(exp_b64.as_bytes());
        let sig_b64 = B64.encode(h.finalize().into_bytes());
        let raw = format!("{VERSION}.{uid_b64}.{exp_b64}.{sig_b64}");
        let err = verify(&raw, &key()).unwrap_err();
        assert!(matches!(err, PendingTotpError::BadSignature));
    }
}
