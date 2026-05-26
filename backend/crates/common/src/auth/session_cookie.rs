//! Opaque session cookie format.
//!
//! BACKEND.md §7.1:
//!
//! ```text
//! tfx_session = v1.<b64url(session_id)>.<b64url(token)>.<b64url(hmac_sha256(key, "v1." + id + "." + token))>
//! ```
//!
//! - `session_id` is the row PK (UUID v7, 16 bytes).
//! - `token` is 32 random bytes from `OsRng`.
//! - The HMAC binds the (version, id, token) triple to the server-side
//!   `AUTH_COOKIE_KEY`, so an attacker forging a cookie cannot get past the
//!   parser — even before we hit the DB.
//! - The DB stores `sha256(token)` only, so a DB dump cannot resurrect a
//!   session.

use base64::Engine;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::RngCore;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use sha2::{Digest, Sha256};
use std::str;
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::ids::SessionId;

pub const COOKIE_NAME: &str = "tfx_session";
const VERSION: &str = "v1";
const TOKEN_BYTES: usize = 32;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum SessionCookieError {
    #[error("malformed session cookie")]
    Malformed,
    #[error("session cookie signature invalid")]
    BadSignature,
    #[error("unsupported session cookie version")]
    UnknownVersion,
    #[error("hmac key invalid: {0}")]
    KeySetup(String),
}

/// Result of issuing a fresh session: the cookie value goes to the browser,
/// the token hash + session id go to the DB row.
pub struct IssuedSession {
    pub cookie_value: String,
    pub session_id: SessionId,
    pub token_hash: [u8; 32],
}

/// Result of parsing+verifying a cookie: the session id and the token hash
/// the SessionsRepo should look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedSession {
    pub session_id: SessionId,
    pub token_hash: [u8; 32],
}

/// Mint a new session cookie. The DB-side row stores `session_id` + `token_hash`;
/// the plaintext token is never persisted.
pub fn issue(key: &SecretString) -> Result<IssuedSession, SessionCookieError> {
    let session_id = SessionId::new();
    let mut token = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut token);

    let cookie_value = encode(session_id, &token, key)?;
    let token_hash = sha256(&token);
    Ok(IssuedSession {
        cookie_value,
        session_id,
        token_hash,
    })
}

/// Parse + HMAC-verify a cookie value. Returns the `session_id` and the
/// `token_hash` to look up in `sessions`. Does NOT touch the DB.
pub fn verify(raw: &str, key: &SecretString) -> Result<VerifiedSession, SessionCookieError> {
    let mut parts = raw.split('.');
    let version = parts.next().ok_or(SessionCookieError::Malformed)?;
    let id_b64 = parts.next().ok_or(SessionCookieError::Malformed)?;
    let token_b64 = parts.next().ok_or(SessionCookieError::Malformed)?;
    let sig_b64 = parts.next().ok_or(SessionCookieError::Malformed)?;
    if parts.next().is_some() {
        return Err(SessionCookieError::Malformed);
    }
    if version != VERSION {
        return Err(SessionCookieError::UnknownVersion);
    }

    let id_bytes = B64
        .decode(id_b64)
        .map_err(|_| SessionCookieError::Malformed)?;
    let token_bytes = B64
        .decode(token_b64)
        .map_err(|_| SessionCookieError::Malformed)?;
    let sig_bytes = B64
        .decode(sig_b64)
        .map_err(|_| SessionCookieError::Malformed)?;

    if id_bytes.len() != 16 || token_bytes.len() != TOKEN_BYTES {
        return Err(SessionCookieError::Malformed);
    }

    // Reconstruct the expected MAC over "v1." + id_b64 + "." + token_b64.
    let expected = mac(key, version, id_b64, token_b64)?;

    // Constant-time compare — never `==` on byte slices for MACs.
    if !bool::from(expected.ct_eq(&sig_bytes)) {
        return Err(SessionCookieError::BadSignature);
    }

    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&id_bytes);
    let session_id = SessionId::from_uuid(Uuid::from_bytes(uuid_bytes));
    let token_hash = sha256(&token_bytes);
    Ok(VerifiedSession {
        session_id,
        token_hash,
    })
}

fn encode(
    session_id: SessionId,
    token: &[u8; TOKEN_BYTES],
    key: &SecretString,
) -> Result<String, SessionCookieError> {
    let id_b64 = B64.encode(session_id.as_uuid().as_bytes());
    let token_b64 = B64.encode(token);
    let mac_bytes = mac(key, VERSION, &id_b64, &token_b64)?;
    let sig_b64 = B64.encode(mac_bytes);
    Ok(format!("{VERSION}.{id_b64}.{token_b64}.{sig_b64}"))
}

fn mac(
    key: &SecretString,
    version: &str,
    id_b64: &str,
    token_b64: &str,
) -> Result<Vec<u8>, SessionCookieError> {
    let key_bytes = key.expose_secret().as_bytes();
    let mut h = HmacSha256::new_from_slice(key_bytes)
        .map_err(|e| SessionCookieError::KeySetup(e.to_string()))?;
    h.update(version.as_bytes());
    h.update(b".");
    h.update(id_b64.as_bytes());
    h.update(b".");
    h.update(token_b64.as_bytes());
    Ok(h.finalize().into_bytes().to_vec())
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(input);
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> SecretString {
        // 32 random bytes, encoded as a string for the test.
        SecretString::from("a-test-key-of-sufficient-length-for-hmac".to_string())
    }

    #[test]
    fn issue_then_verify_round_trip() {
        let k = key();
        let issued = issue(&k).unwrap();
        let parsed = verify(&issued.cookie_value, &k).unwrap();
        assert_eq!(parsed.session_id, issued.session_id);
        assert_eq!(parsed.token_hash, issued.token_hash);
    }

    #[test]
    fn cookie_has_four_dot_parts() {
        let issued = issue(&key()).unwrap();
        assert_eq!(issued.cookie_value.matches('.').count(), 3);
        assert!(issued.cookie_value.starts_with("v1."));
    }

    #[test]
    fn verify_rejects_tampered_token() {
        let k = key();
        let issued = issue(&k).unwrap();
        let mut tampered = issued.cookie_value.clone();
        // Flip the last char of the token segment.
        let parts: Vec<&str> = tampered.split('.').collect();
        let mut new_token = parts[2].to_owned();
        let last_char = new_token.pop().unwrap();
        new_token.push(if last_char == 'A' { 'B' } else { 'A' });
        tampered = format!("{}.{}.{}.{}", parts[0], parts[1], new_token, parts[3]);
        let err = verify(&tampered, &k).unwrap_err();
        assert!(matches!(err, SessionCookieError::BadSignature));
    }

    #[test]
    fn verify_rejects_tampered_signature() {
        let k = key();
        let issued = issue(&k).unwrap();
        let mut tampered = issued.cookie_value.clone();
        // Flip the first char of the signature.
        let parts: Vec<&str> = tampered.split('.').collect();
        let mut new_sig = parts[3].to_owned();
        let first = new_sig.remove(0);
        new_sig.insert(0, if first == 'A' { 'B' } else { 'A' });
        tampered = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], new_sig);
        let err = verify(&tampered, &k).unwrap_err();
        assert!(matches!(err, SessionCookieError::BadSignature));
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let issued = issue(&key()).unwrap();
        let other = SecretString::from("some-other-key-entirely-different".to_string());
        let err = verify(&issued.cookie_value, &other).unwrap_err();
        assert!(matches!(err, SessionCookieError::BadSignature));
    }

    #[test]
    fn verify_rejects_unknown_version() {
        // Hand-craft a v2 cookie.
        let issued = issue(&key()).unwrap();
        let parts: Vec<&str> = issued.cookie_value.splitn(2, '.').collect();
        let v2 = format!("v2.{}", parts[1]);
        let err = verify(&v2, &key()).unwrap_err();
        assert!(matches!(err, SessionCookieError::UnknownVersion));
    }

    #[test]
    fn verify_rejects_malformed_inputs() {
        let k = key();
        for bad in ["", "v1.", "v1.aa.bb", "v1.aa.bb.cc.dd"] {
            let err = verify(bad, &k).unwrap_err();
            assert!(matches!(err, SessionCookieError::Malformed), "{bad:?}");
        }
    }

    #[test]
    fn token_hash_is_not_the_token() {
        let issued = issue(&key()).unwrap();
        // The token doesn't appear in the hash. (Trivially true since the
        // hash is 32 raw bytes, not a string — but pinning the property:
        // the same plaintext token always hashes the same way.)
        let parts: Vec<&str> = issued.cookie_value.split('.').collect();
        let token_bytes = B64.decode(parts[2]).unwrap();
        let h = sha256(&token_bytes);
        assert_eq!(h, issued.token_hash);
    }
}
