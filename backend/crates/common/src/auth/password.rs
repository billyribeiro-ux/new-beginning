//! Argon2id with a keyed pepper.
//!
//! BACKEND.md §1.2 + §7.2: the pepper is passed via `Argon2::new_with_secret`,
//! NOT concatenated onto the password bytes. Concatenation creates a
//! hash-vs-verify asymmetry that ships silently broken; the keyed-secret API
//! binds the pepper into the MAC so verify always reconstructs the same
//! parameters.
//!
//! Both halves of the round-trip live here so the symmetry is impossible to
//! break by accident.

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, SecretString};

/// `Argon2id(m=64 MiB, t=3, p=1)` instances must be built with the pepper as
/// the keyed secret. This helper exists so `hash` and `verify` cannot diverge.
fn argon2_with_pepper(pepper: &[u8]) -> Result<Argon2<'_>, PasswordError> {
    // BACKEND.md §1.2: m = 64 MiB, t = 3, p = 1.
    let params = Params::new(64 * 1024, 3, 1, None)
        .map_err(|e| PasswordError::Setup(format!("argon2 params: {e}")))?;
    Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|e| PasswordError::Setup(format!("argon2 secret: {e}")))
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    /// Misconfigured Argon2 parameters or pepper. A bug, not a user error.
    #[error("argon2 setup: {0}")]
    Setup(String),

    /// Underlying hashing primitive errored. A bug.
    #[error("argon2 hash: {0}")]
    Hash(String),

    /// Stored hash string is unparseable (corrupted row, schema mismatch).
    /// Treated as an internal error — never reveal to the client.
    #[error("argon2 parse: {0}")]
    Parse(String),

    /// `tokio::task::spawn_blocking` join failed (cancelled / panicked).
    #[error("argon2 join: {0}")]
    Join(String),
}

/// Hash a password with the keyed pepper. Runs on a blocking thread.
///
/// The caller is responsible for acquiring the `AppState::hash_semaphore`
/// permit BEFORE invoking — see BACKEND.md §7.2 — so a burst of concurrent
/// logins cannot OOM the host.
pub async fn hash_password(
    password: SecretString,
    pepper: SecretString,
) -> Result<String, PasswordError> {
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret().as_bytes())?;
        let salt = SaltString::generate(&mut OsRng);
        argon
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| PasswordError::Hash(e.to_string()))
    })
    .await
    .map_err(|e| PasswordError::Join(e.to_string()))?
}

/// Verify a password against a stored hash. MUST rebuild the same keyed
/// `Argon2` instance — the pepper is not encoded in `stored`, so a plain
/// `PasswordHash::verify_password(&[Argon2::default()])` would always reject.
///
/// Returns `Ok(true)` on match, `Ok(false)` on mismatch. `Err` is reserved
/// for setup/parse failures, which are bugs.
pub async fn verify_password(
    password: SecretString,
    stored: String,
    pepper: SecretString,
) -> Result<bool, PasswordError> {
    tokio::task::spawn_blocking(move || {
        let argon = argon2_with_pepper(pepper.expose_secret().as_bytes())?;
        let parsed = PasswordHash::new(&stored).map_err(|e| PasswordError::Parse(e.to_string()))?;
        match argon.verify_password(password.expose_secret().as_bytes(), &parsed) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PasswordError::Hash(e.to_string())),
        }
    })
    .await
    .map_err(|e| PasswordError::Join(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pepper() -> SecretString {
        SecretString::from("dev-pepper-do-not-use-in-prod".to_string())
    }

    fn pw(s: &str) -> SecretString {
        SecretString::from(s.to_string())
    }

    #[tokio::test]
    async fn hash_then_verify_round_trip() {
        let h = hash_password(pw("correct horse battery staple"), pepper())
            .await
            .unwrap();
        // Hash format must be the PHC-encoded Argon2id string.
        assert!(h.starts_with("$argon2id$"), "hash format: {h}");
        let ok = verify_password(pw("correct horse battery staple"), h.clone(), pepper())
            .await
            .unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn wrong_password_returns_false() {
        let h = hash_password(pw("correct"), pepper()).await.unwrap();
        let bad = verify_password(pw("wrong"), h, pepper()).await.unwrap();
        assert!(!bad);
    }

    #[tokio::test]
    async fn wrong_pepper_returns_false() {
        // The hash that ships in the DB does not encode the pepper. If the
        // pepper rotates, every existing hash must verify as `false` (forcing
        // re-hash on next login). This test pins that behavior.
        let h = hash_password(pw("password123"), pepper()).await.unwrap();
        let bad = verify_password(
            pw("password123"),
            h,
            SecretString::from("different-pepper".to_string()),
        )
        .await
        .unwrap();
        assert!(!bad, "different pepper must not validate");
    }

    #[tokio::test]
    async fn salts_are_unique_per_hash() {
        let a = hash_password(pw("same-password"), pepper()).await.unwrap();
        let b = hash_password(pw("same-password"), pepper()).await.unwrap();
        assert_ne!(a, b, "identical inputs must produce distinct hashes");
    }
}
