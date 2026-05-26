//! `Money` — i64 cents end-to-end with no silent failure modes.
//!
//! BACKEND.md §5: the newtype's whole job is to make silent money errors
//! impossible. `mul_bps`/`mul_pct` widen to `i128`, divide, then
//! `i64::try_from` — they return `Result`, **never saturate**. Serde guards
//! the JS-safe-integer range symmetrically on both directions so a malformed
//! BFF payload is rejected at the deserialize boundary, not deferred.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Money(i64);

const MAX_JS_SAFE: i64 = (1i64 << 53) - 1;
const MIN_JS_SAFE: i64 = -((1i64 << 53) - 1);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("money arithmetic overflowed i64")]
    Overflow,
    #[error("money value {0} is outside the JS-safe integer range")]
    OutsideJsSafeRange(i64),
}

impl Money {
    pub const ZERO: Money = Money(0);

    #[inline]
    pub const fn from_cents(c: i64) -> Self {
        Money(c)
    }

    #[inline]
    pub const fn cents(self) -> i64 {
        self.0
    }

    /// Multiply by a basis-point factor (1 bps = 0.01 %). Widens to `i128`,
    /// then narrows back with a checked conversion. Returns `Err` on overflow;
    /// **never saturates** — a clamped money value is a silently wrong answer.
    pub fn mul_bps(self, bps: i32) -> Result<Money, MoneyError> {
        let prod: i128 = (self.0 as i128) * (bps as i128);
        let scaled: i128 = prod / 10_000;
        i64::try_from(scaled)
            .map(Money)
            .map_err(|_| MoneyError::Overflow)
    }

    /// Multiply by a whole-percent factor. Defined in terms of `mul_bps`.
    pub fn mul_pct(self, pct: i32) -> Result<Money, MoneyError> {
        let bps = pct.checked_mul(100).ok_or(MoneyError::Overflow)?;
        self.mul_bps(bps)
    }

    /// Multiply by a non-negative integer quantity (e.g. line-item rollup).
    /// Same `i128`-widening discipline; returns `Err` on overflow.
    pub fn mul_qty(self, qty: i64) -> Result<Money, MoneyError> {
        let prod: i128 = (self.0 as i128) * (qty as i128);
        i64::try_from(prod)
            .map(Money)
            .map_err(|_| MoneyError::Overflow)
    }

    pub fn checked_add(self, r: Money) -> Option<Money> {
        self.0.checked_add(r.0).map(Money)
    }

    pub fn checked_sub(self, r: Money) -> Option<Money> {
        self.0.checked_sub(r.0).map(Money)
    }

    /// Reject values that would silently lose precision once they cross into
    /// the JS `Number` domain on the BFF side.
    fn ensure_js_safe(v: i64) -> Result<i64, MoneyError> {
        if (MIN_JS_SAFE..=MAX_JS_SAFE).contains(&v) {
            Ok(v)
        } else {
            Err(MoneyError::OutsideJsSafeRange(v))
        }
    }
}

// Operator impls exist for ergonomics in test code and bounded-input rollups;
// production call sites should prefer `checked_add` / `checked_sub` /
// `mul_qty` / `mul_bps` which return `Result`. The unchecked ops below panic
// on overflow in debug and wrap in release — matching std `i64` semantics.
impl Add for Money {
    type Output = Money;
    fn add(self, r: Money) -> Money {
        Money(self.0 + r.0)
    }
}
impl Sub for Money {
    type Output = Money;
    fn sub(self, r: Money) -> Money {
        Money(self.0 - r.0)
    }
}
impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(-self.0)
    }
}

// Deliberately NO `Mul<i64>`, NO `Mul<Money>`. Quantity-times-price goes
// through `Money::mul_qty` which widens to i128 and returns `Result`.

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        let abs = self.0.unsigned_abs();
        write!(
            f,
            "{}${}.{:02}",
            if neg { "-" } else { "" },
            abs / 100,
            abs % 100
        )
    }
}

// Symmetric JS-safety guard: reject out-of-range on BOTH directions.
impl Serialize for Money {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Money::ensure_js_safe(self.0).map_err(serde::ser::Error::custom)?;
        s.serialize_i64(self.0)
    }
}
impl<'de> Deserialize<'de> for Money {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = i64::deserialize(d)?;
        Money::ensure_js_safe(v)
            .map(Money)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_cents_as_dollars() {
        assert_eq!(Money::from_cents(99_700).to_string(), "$997.00");
        assert_eq!(Money::from_cents(7).to_string(), "$0.07");
        assert_eq!(Money::from_cents(0).to_string(), "$0.00");
        assert_eq!(Money::from_cents(-1_205).to_string(), "-$12.05");
    }

    #[test]
    fn mul_bps_basic_cases() {
        // 8 % sales tax on $100.00 = $8.00
        assert_eq!(
            Money::from_cents(10_000).mul_bps(800),
            Ok(Money::from_cents(800))
        );
        // 0 bps is always zero
        assert_eq!(
            Money::from_cents(1_234_567).mul_bps(0),
            Ok(Money::from_cents(0))
        );
        // 10_000 bps == 100 % == identity
        assert_eq!(
            Money::from_cents(1_234_567).mul_bps(10_000),
            Ok(Money::from_cents(1_234_567))
        );
    }

    #[test]
    fn mul_bps_overflow_is_err_not_saturated() {
        // The exact case BACKEND.md §25 item 17 mandates:
        // i64::MAX cents at 200 % overflows i64; must surface as Err.
        let result = Money::from_cents(i64::MAX).mul_pct(200);
        assert_eq!(result, Err(MoneyError::Overflow));
    }

    #[test]
    fn mul_pct_overflow_in_pct_to_bps() {
        // pct.checked_mul(100) overflows before the i128 widening.
        assert_eq!(
            Money::from_cents(1).mul_pct(i32::MAX),
            Err(MoneyError::Overflow)
        );
    }

    #[test]
    fn mul_qty_overflow_is_err() {
        assert_eq!(
            Money::from_cents(i64::MAX).mul_qty(2),
            Err(MoneyError::Overflow)
        );
        assert_eq!(
            Money::from_cents(50_000).mul_qty(3),
            Ok(Money::from_cents(150_000))
        );
    }

    #[test]
    fn checked_add_sub_surface_overflow() {
        assert_eq!(
            Money::from_cents(i64::MAX).checked_add(Money::from_cents(1)),
            None
        );
        assert_eq!(
            Money::from_cents(i64::MIN).checked_sub(Money::from_cents(1)),
            None
        );
        assert_eq!(
            Money::from_cents(100).checked_add(Money::from_cents(50)),
            Some(Money::from_cents(150))
        );
    }

    #[test]
    fn serde_round_trip_in_range() {
        let m = Money::from_cents(99_700);
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "99700");
        let back: Money = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn serialize_rejects_above_js_safe() {
        let m = Money::from_cents(MAX_JS_SAFE + 1);
        let err = serde_json::to_string(&m).unwrap_err();
        assert!(
            err.to_string().contains("JS-safe"),
            "expected JS-safe error, got: {err}"
        );
    }

    #[test]
    fn serialize_rejects_below_js_safe() {
        let m = Money::from_cents(MIN_JS_SAFE - 1);
        let err = serde_json::to_string(&m).unwrap_err();
        assert!(err.to_string().contains("JS-safe"));
    }

    #[test]
    fn deserialize_rejects_above_js_safe() {
        // A JSON integer above 2^53-1: malformed BFF payload.
        let too_big = format!("{}", MAX_JS_SAFE + 1);
        let err: serde_json::Error = serde_json::from_str::<Money>(&too_big).unwrap_err();
        assert!(err.to_string().contains("JS-safe"));
    }

    #[test]
    fn deserialize_rejects_below_js_safe() {
        let too_small = format!("{}", MIN_JS_SAFE - 1);
        let err: serde_json::Error = serde_json::from_str::<Money>(&too_small).unwrap_err();
        assert!(err.to_string().contains("JS-safe"));
    }

    #[test]
    fn deserialize_accepts_boundary() {
        let max: Money = serde_json::from_str(&MAX_JS_SAFE.to_string()).unwrap();
        let min: Money = serde_json::from_str(&MIN_JS_SAFE.to_string()).unwrap();
        assert_eq!(max.cents(), MAX_JS_SAFE);
        assert_eq!(min.cents(), MIN_JS_SAFE);
    }

    #[test]
    fn zero_constant_matches() {
        assert_eq!(Money::ZERO.cents(), 0);
        assert_eq!(Money::ZERO, Money::from_cents(0));
    }
}
