//! XChaCha20-Poly1305 encryption for TOTP secrets.
//!
//! BACKEND.md §1.2 / §7.3: the 20-byte TOTP secret is encrypted at rest with
//! `AUTH_TOTP_KEY` so a DB dump alone cannot replay 2FA codes.
//!
//! On-disk layout in `users.totp_secret_encrypted` (BYTEA, 60 bytes total):
//!
//! ```text
//! [nonce: 24 bytes][ciphertext+tag: ciphertext (20 bytes) || poly1305 tag (16)]
//! ```
//!
//! Nonce is freshly random per encryption; XChaCha20-Poly1305 makes a 24-byte
//! random nonce safe (the wider nonce space removes the birthday-bound on
//! plain ChaCha20).

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use secrecy::{ExposeSecret, SecretBox, SecretString};

/// Boxed-secret wrapper for raw byte secrets. Aliased so the spelling
/// matches BACKEND.md §7.3's text ("SecretVec") even though `secrecy` 0.10
/// renamed it.
pub type SecretBytes = SecretBox<Vec<u8>>;

const NONCE_BYTES: usize = 24;
const TAG_BYTES: usize = 16;

/// Raw 20-byte TOTP secret. Wrapped in `SecretBox` so it never lands in
/// `Debug` output / logs.
pub fn random_secret() -> SecretBytes {
    use rand::RngCore;
    let mut buf = vec![0u8; 20];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    SecretBox::new(Box::new(buf))
}

#[derive(Debug, thiserror::Error)]
pub enum TotpAtRestError {
    #[error("AUTH_TOTP_KEY must be 32 bytes (got {0})")]
    KeyLen(usize),
    #[error("ciphertext too short: {0} bytes")]
    CiphertextLen(usize),
    #[error("decryption failed (wrong key, tampered, or corrupted ciphertext)")]
    Decrypt,
    #[error("encryption failed")]
    Encrypt,
}

fn parse_key(key: &SecretString) -> Result<Key, TotpAtRestError> {
    let raw = key.expose_secret().as_bytes();
    if raw.len() != 32 {
        return Err(TotpAtRestError::KeyLen(raw.len()));
    }
    Ok(*Key::from_slice(raw))
}

/// Encrypt a 20-byte TOTP secret. Returns `[nonce][ciphertext+tag]`.
pub fn encrypt(secret: &SecretBytes, key: &SecretString) -> Result<Vec<u8>, TotpAtRestError> {
    let key = parse_key(key)?;
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, secret.expose_secret().as_slice())
        .map_err(|_| TotpAtRestError::Encrypt)?;

    let mut out = Vec::with_capacity(NONCE_BYTES + ciphertext.len());
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt the BYTEA blob produced by `encrypt`.
pub fn decrypt(blob: &[u8], key: &SecretString) -> Result<SecretBytes, TotpAtRestError> {
    if blob.len() < NONCE_BYTES + TAG_BYTES {
        return Err(TotpAtRestError::CiphertextLen(blob.len()));
    }
    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_BYTES);
    let key = parse_key(key)?;
    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| TotpAtRestError::Decrypt)?;
    Ok(SecretBox::new(Box::new(plaintext)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> SecretString {
        // exactly 32 bytes
        SecretString::from("0123456789abcdef0123456789abcdef".to_string())
    }

    #[test]
    fn round_trip() {
        let secret = random_secret();
        let blob = encrypt(&secret, &key()).unwrap();
        let recovered = decrypt(&blob, &key()).unwrap();
        assert_eq!(recovered.expose_secret(), secret.expose_secret());
    }

    #[test]
    fn ciphertext_bytes_change_per_call_due_to_random_nonce() {
        let secret = random_secret();
        let a = encrypt(&secret, &key()).unwrap();
        let b = encrypt(&secret, &key()).unwrap();
        assert_ne!(a, b, "random nonce must produce distinct ciphertext");
    }

    #[test]
    fn wrong_key_is_decrypt_error_not_silent_garbage() {
        let secret = random_secret();
        let blob = encrypt(&secret, &key()).unwrap();
        let other = SecretString::from("ffffffffffffffffffffffffffffffff".to_string());
        let err = decrypt(&blob, &other).unwrap_err();
        assert!(matches!(err, TotpAtRestError::Decrypt));
    }

    #[test]
    fn tampered_ciphertext_is_decrypt_error() {
        let secret = random_secret();
        let mut blob = encrypt(&secret, &key()).unwrap();
        let i = NONCE_BYTES + 2;
        blob[i] ^= 0xff;
        let err = decrypt(&blob, &key()).unwrap_err();
        assert!(matches!(err, TotpAtRestError::Decrypt));
    }

    #[test]
    fn key_must_be_32_bytes() {
        let short = SecretString::from("short".to_string());
        let secret = random_secret();
        let err = encrypt(&secret, &short).unwrap_err();
        assert!(matches!(err, TotpAtRestError::KeyLen(5)));
    }

    #[test]
    fn truncated_blob_is_length_error() {
        let err = decrypt(&[0u8; 5], &key()).unwrap_err();
        assert!(matches!(err, TotpAtRestError::CiphertextLen(_)));
    }
}
