//! Serde helpers shared across HTTP handler bodies.
//!
//! [`double_option`] distinguishes "field absent" (no change) from "field
//! present and null" (clear the column) in a JSON PATCH body. The default
//! serde behavior collapses both cases to `None`, which is unusable for
//! tri-state fields like `headline` or `dnd_start`.

use serde::{Deserialize, Deserializer};

pub fn double_option<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Some)
}
