//! Serde helpers for ZUID (64-bit integer IDs)
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
            .ok_or_else(|| serde::de::Error::custom("invalid zuid number")),
        _ => Err(serde::de::Error::custom("zuid must be a string or number")),
    }
}

/// Serialize an `Option<i64>` as a JSON string or null.
pub fn serialize_opt<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serialize(v, serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserialize an `Option<i64>` from either a JSON string, number, or null.
pub fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("invalid zuid number")),
        Some(_) => Err(serde::de::Error::custom("zuid must be a string or number")),
        None => Ok(None),
    }
}

/// Submodule usable with `#[serde(with = "common::serde_zuid::opt")]` on
/// `Option<i64>` fields (serde requires a module exposing `serialize`/`deserialize`).
pub mod opt {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_opt(value, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_opt(deserializer)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json;

    /// Wrapper struct for testing the serde helpers via `#[serde(with = …)]`.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(transparent)]
    struct Zuid(#[serde(with = "super")] i64);

    #[test]
    fn test_serialize_produces_string() {
        let zuid = Zuid(1234567890123456789i64);
        let json = serde_json::to_string(&zuid).unwrap();
        assert_eq!(json, r#""1234567890123456789""#);
    }

    #[test]
    fn test_deserialize_from_string() {
        let zuid: Zuid = serde_json::from_str(r#""42""#).unwrap();
        assert_eq!(zuid, Zuid(42));
    }

    #[test]
    fn test_deserialize_from_number() {
        let zuid: Zuid = serde_json::from_str(r#"42"#).unwrap();
        assert_eq!(zuid, Zuid(42));
    }

    #[test]
    fn test_deserialize_large_number() {
        let zuid: Zuid = serde_json::from_str(r#""1234567890123456789""#).unwrap();
        assert_eq!(zuid, Zuid(1234567890123456789i64));
    }

    #[test]
    fn test_round_trip() {
        let original = Zuid(-9223372036854775808i64); // i64::MIN
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Zuid = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_round_trip_positive() {
        let original = Zuid(9223372036854775807i64); // i64::MAX
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Zuid = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_deserialize_from_negative_number() {
        let zuid: Zuid = serde_json::from_str(r#"-1"#).unwrap();
        assert_eq!(zuid, Zuid(-1));
    }

    #[test]
    fn test_deserialize_invalid_type() {
        let result: Result<Zuid, _> = serde_json::from_str(r#"true"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_invalid_string() {
        let result: Result<Zuid, _> = serde_json::from_str(r#""not-a-number""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_float_rejected() {
        let result: Result<Zuid, _> = serde_json::from_str(r#"3.14"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_zero() {
        let zuid: Zuid = serde_json::from_str(r#"0"#).unwrap();
        assert_eq!(zuid, Zuid(0));
    }

    #[test]
    fn test_deserialize_zero_string() {
        let zuid: Zuid = serde_json::from_str(r#""0""#).unwrap();
        assert_eq!(zuid, Zuid(0));
    }
}
