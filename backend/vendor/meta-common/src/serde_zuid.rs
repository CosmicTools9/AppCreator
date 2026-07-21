//! Serde helpers for ZUID / CRC64 (64-bit integer IDs)
//!
//! Serializes `i64` values as JSON strings to avoid JavaScript number precision loss.
//! Deserializes from either JSON strings or numbers for backward compatibility.

use serde::{Deserialize, Deserializer, Serializer};

/// Serialize an `i64` as a JSON string.
pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// Deserialize an `i64` from either a JSON string or number.
pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => s.parse().map_err(serde::de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("invalid 64-bit id number")),
        _ => Err(serde::de::Error::custom(
            "64-bit id must be a string or number",
        )),
    }
}
