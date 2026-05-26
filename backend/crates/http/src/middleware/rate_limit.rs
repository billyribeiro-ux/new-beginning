//! Generalized per-IP rate-limit framework.
//!
//! BACKEND.md §12. PR #3 introduced a one-off `LoginLimiter`; PR #6
//! generalizes it into a small bucket framework that every public/authed
//! surface picks from. Each `Bucket` owns its own `governor::RateLimiter`
//! so the buckets cannot accidentally cross-pollute each other's quotas.
//!
//! BACKEND.md §23 risk #2: single-instance only. The whole module sits
//! behind one type (`RateLimiterSet`), so when Redis-backed limiting lands
//! every call site is unchanged — we swap the impl.

use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

type IpLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

/// Named bucket per surface. The values mirror BACKEND.md §12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bucket {
    /// `GET /v1/public/*` — 60/min/IP.
    PublicRead,
    /// `POST /v1/public/leads` — 5/min/IP.
    LeadCapture,
    /// `POST /v1/public/contact` — 3/min/IP.
    Contact,
    /// `/v1/auth/login` + `/v1/auth/login/totp` — 5/min/IP.
    /// Held intentionally tight; raises to 10/min when the
    /// `(IP, normalized_email)` key-tuple lands.
    Login,
}

impl Bucket {
    fn quota_per_minute(self) -> u32 {
        match self {
            Bucket::PublicRead => 60,
            Bucket::LeadCapture => 5,
            Bucket::Contact => 3,
            Bucket::Login => 5,
        }
    }

    fn quota(self) -> Quota {
        let per_minute = NonZeroU32::new(self.quota_per_minute()).expect("non-zero quota");
        // Burst = quota — same shape as PR #3 (a steady drip, not bursty).
        Quota::per_minute(per_minute).allow_burst(per_minute)
    }

    /// `snake_case` label used by metrics + tracing. Bounded set, safe for
    /// `metrics` labels per BACKEND.md §15 rule.
    pub fn as_label(self) -> &'static str {
        match self {
            Bucket::PublicRead => "public_read",
            Bucket::LeadCapture => "lead_capture",
            Bucket::Contact => "contact",
            Bucket::Login => "login",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimited {
    pub retry_after_secs: u32,
}

/// One `governor` instance per bucket. Cheap to clone via `Arc`.
#[derive(Clone)]
pub struct RateLimiterSet {
    public_read: Arc<IpLimiter>,
    lead_capture: Arc<IpLimiter>,
    contact: Arc<IpLimiter>,
    login: Arc<IpLimiter>,
    clock: DefaultClock,
}

impl Default for RateLimiterSet {
    fn default() -> Self {
        Self {
            public_read: Arc::new(RateLimiter::keyed(Bucket::PublicRead.quota())),
            lead_capture: Arc::new(RateLimiter::keyed(Bucket::LeadCapture.quota())),
            contact: Arc::new(RateLimiter::keyed(Bucket::Contact.quota())),
            login: Arc::new(RateLimiter::keyed(Bucket::Login.quota())),
            clock: DefaultClock::default(),
        }
    }
}

impl RateLimiterSet {
    /// Consume one token from `bucket` for `ip`. Returns `Err(RateLimited)`
    /// with a positive (≥ 1) `retry_after_secs` when the bucket is empty.
    /// Handlers map this directly to `AppError::RateLimited`.
    pub fn check(&self, bucket: Bucket, ip: IpAddr) -> Result<(), RateLimited> {
        let limiter: &IpLimiter = match bucket {
            Bucket::PublicRead => &self.public_read,
            Bucket::LeadCapture => &self.lead_capture,
            Bucket::Contact => &self.contact,
            Bucket::Login => &self.login,
        };
        match limiter.check_key(&ip) {
            Ok(()) => Ok(()),
            Err(not_until) => {
                let secs = not_until.wait_time_from(self.clock.now()).as_secs() as u32;
                Err(RateLimited {
                    retry_after_secs: secs.max(1),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip() -> IpAddr {
        "203.0.113.42".parse().unwrap()
    }

    #[test]
    fn lead_capture_fires_on_sixth_attempt() {
        let limiter = RateLimiterSet::default();
        for i in 1..=5 {
            assert!(
                limiter.check(Bucket::LeadCapture, ip()).is_ok(),
                "attempt {i} should pass"
            );
        }
        let err = limiter.check(Bucket::LeadCapture, ip()).unwrap_err();
        assert!(err.retry_after_secs >= 1);
    }

    #[test]
    fn buckets_are_independent_per_ip() {
        let limiter = RateLimiterSet::default();
        // Exhaust the contact bucket.
        for _ in 0..3 {
            limiter.check(Bucket::Contact, ip()).unwrap();
        }
        assert!(limiter.check(Bucket::Contact, ip()).is_err());
        // PublicRead bucket on the SAME ip is unaffected.
        assert!(limiter.check(Bucket::PublicRead, ip()).is_ok());
    }

    #[test]
    fn buckets_are_independent_per_ip_keying() {
        let limiter = RateLimiterSet::default();
        let a: IpAddr = "203.0.113.1".parse().unwrap();
        let b: IpAddr = "203.0.113.2".parse().unwrap();
        for _ in 0..3 {
            limiter.check(Bucket::Contact, a).unwrap();
        }
        // a is now spent.
        assert!(limiter.check(Bucket::Contact, a).is_err());
        // b is fresh.
        assert!(limiter.check(Bucket::Contact, b).is_ok());
    }

    #[test]
    fn labels_are_static_snake_case_and_distinct() {
        let labels = [
            Bucket::PublicRead.as_label(),
            Bucket::LeadCapture.as_label(),
            Bucket::Contact.as_label(),
            Bucket::Login.as_label(),
        ];
        for l in labels {
            assert!(l.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
        }
        let mut set = std::collections::HashSet::new();
        for l in labels {
            assert!(set.insert(l), "duplicate label: {l}");
        }
    }
}
