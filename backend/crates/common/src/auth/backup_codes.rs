//! Backup codes — 10-char Crockford base32, Argon2id-hashed at rest.
//!
//! BACKEND.md §1.2 / §7.3. Why Crockford:
//! - Drops the visually-ambiguous chars `I`, `L`, `O`, `0`, `U` from the
//!   alphabet (RFC 4648 base32 keeps them) — important because users type
//!   these manually under stress.
//! - 32 chars per slot = 5 bits per slot → 10 slots = 50 bits of entropy.
//!   With only 10 attempts allowed (the code count itself), this is plenty.
//!
//! Hashing 10 codes at issuance costs ~3 s wall-clock through the
//! `argon2_with_pepper` builder (BACKEND.md §7.3 — same instance as password
//! hashing). That's a known, accepted cost for a user-initiated one-off.

use argon2::password_hash::{rand_core::OsRng as Argon2OsRng, SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};

pub const CODE_COUNT: usize = 10;
pub const CODE_LEN: usize = 10;

/// Crockford base32 alphabet. Excludes `I`, `L`, `O`, `0`, `U` to dodge
/// transcription ambiguity. Length is 32 so each char encodes 5 bits.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[derive(Debug, thiserror::Error)]
pub enum BackupCodesError {
    #[error("argon2 setup: {0}")]
    Setup(String),
    #[error("argon2 hash: {0}")]
    Hash(String),
    #[error("argon2 parse: {0}")]
    Parse(String),
}

fn argon2_with_pepper(pepper: &[u8]) -> Result<Argon2<'_>, BackupCodesError> {
    let params = Params::new(64 * 1024, 3, 1, None)
        .map_err(|e| BackupCodesError::Setup(format!("params: {e}")))?;
    Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|e| BackupCodesError::Setup(format!("secret: {e}")))
}

/// Generate `CODE_COUNT` codes of length `CODE_LEN`. The plaintext is shown
/// to the user EXACTLY ONCE; we keep only the hashes.
pub fn generate_plaintext() -> Vec<SecretString> {
    let mut out = Vec::with_capacity(CODE_COUNT);
    for _ in 0..CODE_COUNT {
        let mut buf = [0u8; CODE_LEN];
        OsRng.fill_bytes(&mut buf);
        let code: String = buf
            .iter()
            .map(|b| ALPHABET[(*b as usize) % 32] as char)
            .collect();
        out.push(SecretString::from(code));
    }
    out
}

/// Hash `plaintext` with the keyed pepper. Returns the PHC-encoded Argon2id
/// hash as UTF-8 bytes — ready for the `users.twofa_backup_codes_hash`
/// `BYTEA[]` column.
///
/// MUST be called under the `AppState::hash_semaphore` permit.
pub async fn hash_one(
    plaintext: SecretString,
    pepper: SecretString,
) -> Result<Vec<u8>, BackupCodesError> {
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret().as_bytes())?;
        let salt = SaltString::generate(&mut Argon2OsRng);
        argon
            .hash_password(plaintext.expose_secret().as_bytes(), &salt)
            .map(|h| h.to_string().into_bytes())
            .map_err(|e| BackupCodesError::Hash(e.to_string()))
    })
    .await
    .map_err(|e| BackupCodesError::Hash(format!("join: {e}")))?
}

/// Verify `candidate` against `stored` (one slot's PHC bytes). Returns:
/// - `Ok(true)` on match (caller MUST then zero the slot in the DB)
/// - `Ok(false)` on mismatch
/// - `Err(_)` on parse/setup failure (a bug, never expose to client)
///
/// A consumed slot is all-zero `BYTEA` — those bytes are not a valid PHC
/// string, so this function returns `Ok(false)` for them (the parse fails
/// in a way we treat as "no match", not as an error).
pub async fn verify_one(
    candidate: SecretString,
    stored: Vec<u8>,
    pepper: SecretString,
) -> Result<bool, BackupCodesError> {
    if is_zeroed_slot(&stored) {
        return Ok(false);
    }
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret().as_bytes())?;
        let stored_str =
            std::str::from_utf8(&stored).map_err(|e| BackupCodesError::Parse(e.to_string()))?;
        let parsed =
            PasswordHash::new(stored_str).map_err(|e| BackupCodesError::Parse(e.to_string()))?;
        match argon.verify_password(candidate.expose_secret().as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(BackupCodesError::Hash(e.to_string())),
        }
    })
    .await
    .map_err(|e| BackupCodesError::Hash(format!("join: {e}")))?
}

/// A consumed-slot marker: all-zero BYTEA preserves array length so the UI
/// can show "X/10 remaining" (BACKEND.md §7.3).
pub fn zeroed_slot() -> Vec<u8> {
    // Any length works; pick 1 to keep the column small.
    vec![0u8]
}

pub fn is_zeroed_slot(b: &[u8]) -> bool {
    !b.is_empty() && b.iter().all(|&x| x == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pepper() -> SecretString {
        SecretString::from("dev-pepper-do-not-use-in-prod".to_string())
    }

    #[test]
    fn generate_returns_ten_distinct_uppercase_alphanumeric_codes() {
        let codes = generate_plaintext();
        assert_eq!(codes.len(), CODE_COUNT);
        let mut seen = std::collections::HashSet::new();
        for c in &codes {
            let s = c.expose_secret();
            assert_eq!(s.len(), CODE_LEN, "code length: {s}");
            assert!(s.chars().all(|ch| ALPHABET.contains(&(ch as u8))));
            // Disallow the ambiguous Crockford-excluded chars.
            for bad in ['I', 'L', 'O', 'U'] {
                assert!(!s.contains(bad), "code {s} contains ambiguous char {bad}");
            }
            seen.insert(s.to_string());
        }
        assert_eq!(seen.len(), CODE_COUNT, "codes must be distinct");
    }

    #[tokio::test]
    async fn hash_then_verify_round_trip() {
        let plaintext = SecretString::from("ABCDEFGHJK".to_string());
        let stored = hash_one(plaintext.clone(), pepper()).await.unwrap();
        let ok = verify_one(plaintext, stored.clone(), pepper())
            .await
            .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn wrong_code_returns_false() {
        let plaintext = SecretString::from("ABCDEFGHJK".to_string());
        let stored = hash_one(plaintext, pepper()).await.unwrap();
        let bad = verify_one(
            SecretString::from("ZZZZZZZZZZ".to_string()),
            stored,
            pepper(),
        )
        .await
        .unwrap();
        assert!(!bad);
    }

    #[tokio::test]
    async fn zeroed_slot_never_validates() {
        let any = SecretString::from("ABCDEFGHJK".to_string());
        let consumed = zeroed_slot();
        let ok = verify_one(any, consumed, pepper()).await.unwrap();
        assert!(!ok);
    }

    #[test]
    fn is_zeroed_slot_excludes_empty() {
        assert!(!is_zeroed_slot(&[]));
        assert!(is_zeroed_slot(&[0]));
        assert!(is_zeroed_slot(&[0, 0, 0]));
        assert!(!is_zeroed_slot(&[0, 1, 0]));
    }
}
