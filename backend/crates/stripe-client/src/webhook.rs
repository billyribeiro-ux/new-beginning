//! Stripe webhook signature verification.
//!
//! Spec: <https://docs.stripe.com/webhooks#verify-manually>
//!
//! ```text
//! Stripe-Signature: t=1614265564,v1=HEX_SHA256(t.payload),v0=...
//! ```
//!
//! - Compute `HMAC-SHA256(secret, "{t}.{raw_body}")`, hex-encoded.
//! - Constant-time-compare against the `v1` field. Stripe rotates with
//!   `v2` someday; we accept only `v1` here and document.
//! - Reject when `now() - t > tolerance` (default 5 min) — defends against
//!   replay of an old payload.

use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;
const DEFAULT_TOLERANCE: Duration = Duration::from_secs(300);

#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("missing Stripe-Signature header")]
    MissingHeader,
    #[error("malformed Stripe-Signature header")]
    Malformed,
    #[error("Stripe-Signature timestamp outside tolerance")]
    Stale,
    #[error("Stripe-Signature signature mismatch")]
    BadSignature,
    #[error("payload was not valid UTF-8")]
    NonUtf8,
    #[error("payload was not valid JSON: {0}")]
    BadJson(String),
    #[error("hmac setup: {0}")]
    KeySetup(String),
    #[error("system clock failure")]
    Clock,
}

/// The fields PR #7's webhook handler needs. We deliberately don't reflect
/// the full event payload here — only `id` + `type` cross the wrapper
/// boundary. The raw JSON stays attached so the handler module can downcast
/// per event type later.
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub id: String,
    pub kind: String,
    pub raw: serde_json::Value,
}

#[derive(Deserialize)]
struct EnvelopeHead {
    id: String,
    #[serde(rename = "type")]
    kind: String,
}

/// Verify the signature header against `raw_body` using `secret`, parse the
/// body as JSON, and return the (id, type, raw) triple.
pub fn verify_signature(
    raw_body: &[u8],
    signature_header: Option<&str>,
    secret: &SecretString,
) -> Result<ParsedEvent, SignatureError> {
    verify_signature_at(
        raw_body,
        signature_header,
        secret,
        now_unix()?,
        DEFAULT_TOLERANCE,
    )
}

pub fn verify_signature_at(
    raw_body: &[u8],
    signature_header: Option<&str>,
    secret: &SecretString,
    now_unix_secs: i64,
    tolerance: Duration,
) -> Result<ParsedEvent, SignatureError> {
    let header = signature_header.ok_or(SignatureError::MissingHeader)?;
    let (t, v1) = parse_header(header)?;

    let drift = (now_unix_secs - t).unsigned_abs();
    if drift > tolerance.as_secs() {
        return Err(SignatureError::Stale);
    }

    let body_str = std::str::from_utf8(raw_body).map_err(|_| SignatureError::NonUtf8)?;
    let signed_payload = format!("{t}.{body_str}");

    let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes())
        .map_err(|e| SignatureError::KeySetup(e.to_string()))?;
    mac.update(signed_payload.as_bytes());
    let expected = mac.finalize().into_bytes();

    let received = hex::decode(v1).map_err(|_| SignatureError::Malformed)?;
    if !bool::from(expected.as_slice().ct_eq(&received)) {
        return Err(SignatureError::BadSignature);
    }

    let raw: serde_json::Value =
        serde_json::from_slice(raw_body).map_err(|e| SignatureError::BadJson(e.to_string()))?;
    let head: EnvelopeHead =
        serde_json::from_value(raw.clone()).map_err(|e| SignatureError::BadJson(e.to_string()))?;
    Ok(ParsedEvent {
        id: head.id,
        kind: head.kind,
        raw,
    })
}

fn parse_header(header: &str) -> Result<(i64, String), SignatureError> {
    let mut t: Option<i64> = None;
    let mut v1: Option<String> = None;
    for part in header.split(',') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next().ok_or(SignatureError::Malformed)?.trim();
        let v = kv.next().ok_or(SignatureError::Malformed)?.trim();
        match k {
            "t" => t = v.parse().ok(),
            "v1" => v1 = Some(v.to_string()),
            // Stripe also sends v0 (test mode replay) and may add v2 later.
            // Ignored for now — see SignatureError doc.
            _ => {}
        }
    }
    match (t, v1) {
        (Some(t), Some(v1)) => Ok((t, v1)),
        _ => Err(SignatureError::Malformed),
    }
}

fn now_unix() -> Result<i64, SignatureError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|_| SignatureError::Clock)
}

/// Helper for tests + the evidence script — produce the `Stripe-Signature`
/// header value for a given (timestamp, body, secret) triple. **Test-only**;
/// do NOT ship a route that returns this.
pub fn sign_for_test(raw_body: &[u8], unix_ts: i64, secret: &SecretString) -> String {
    let body_str = std::str::from_utf8(raw_body).expect("test body must be utf-8");
    let signed = format!("{unix_ts}.{body_str}");
    let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes()).unwrap();
    mac.update(signed.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("t={unix_ts},v1={sig}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> SecretString {
        SecretString::from("whsec_test_pr7_evidence_secret".to_string())
    }

    fn body(id: &str, kind: &str) -> Vec<u8> {
        format!(r#"{{"id":"{id}","type":"{kind}","data":{{}}}}"#).into_bytes()
    }

    #[test]
    fn round_trip() {
        let raw = body("evt_1", "checkout.session.completed");
        let ts = 1_700_000_000_i64;
        let header = sign_for_test(&raw, ts, &secret());
        let parsed = verify_signature_at(&raw, Some(&header), &secret(), ts, DEFAULT_TOLERANCE)
            .expect("verify");
        assert_eq!(parsed.id, "evt_1");
        assert_eq!(parsed.kind, "checkout.session.completed");
    }

    #[test]
    fn missing_header() {
        let raw = body("evt_1", "foo");
        let err = verify_signature_at(&raw, None, &secret(), 1, DEFAULT_TOLERANCE).unwrap_err();
        assert!(matches!(err, SignatureError::MissingHeader));
    }

    #[test]
    fn tampered_body_rejected() {
        let raw = body("evt_1", "foo");
        let ts = 1_700_000_000_i64;
        let header = sign_for_test(&raw, ts, &secret());
        let mut tampered = raw.clone();
        let i = tampered.len() / 2;
        tampered[i] ^= 0x01;
        let err = verify_signature_at(&tampered, Some(&header), &secret(), ts, DEFAULT_TOLERANCE)
            .unwrap_err();
        // Tampering may turn it into invalid JSON OR a bad sig — both are
        // failures that reject the event. The exact discriminant is fragile
        // (depends on which byte we flipped); both Malformed / BadSignature /
        // BadJson are acceptable here.
        assert!(matches!(
            err,
            SignatureError::BadSignature | SignatureError::BadJson(_) | SignatureError::NonUtf8
        ));
    }

    #[test]
    fn wrong_secret_rejected() {
        let raw = body("evt_1", "foo");
        let ts = 1_700_000_000_i64;
        let header = sign_for_test(&raw, ts, &secret());
        let other = SecretString::from("whsec_test_OTHER".to_string());
        let err =
            verify_signature_at(&raw, Some(&header), &other, ts, DEFAULT_TOLERANCE).unwrap_err();
        assert!(matches!(err, SignatureError::BadSignature));
    }

    #[test]
    fn stale_timestamp_rejected() {
        let raw = body("evt_1", "foo");
        let ts = 1_700_000_000_i64;
        let header = sign_for_test(&raw, ts, &secret());
        // "now" is 6 minutes after the signed timestamp.
        let now = ts + 6 * 60;
        let err = verify_signature_at(&raw, Some(&header), &secret(), now, DEFAULT_TOLERANCE)
            .unwrap_err();
        assert!(matches!(err, SignatureError::Stale));
    }

    #[test]
    fn malformed_header_rejected() {
        let raw = body("evt_1", "foo");
        let err = verify_signature_at(&raw, Some("garbage"), &secret(), 1, DEFAULT_TOLERANCE)
            .unwrap_err();
        assert!(matches!(err, SignatureError::Malformed));
    }
}
