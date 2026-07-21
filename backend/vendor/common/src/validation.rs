//! PostgreSQL identifier validation utilities
//!
//! Prevents SQL injection in DDL commands where identifiers cannot be parameterized.

/// Validate a PostgreSQL identifier to prevent SQL injection.
///
/// Rules:
/// - Must not be empty
/// - Must not exceed 63 bytes (PostgreSQL NAMEDATALEN - 1)
/// - Must start with a letter or underscore
/// - Must contain only ASCII alphanumeric characters and underscores
///
/// # Examples
/// ```
/// use common::validate_pg_ident;
/// assert!(validate_pg_ident("my_table").is_ok());
/// assert!(validate_pg_ident("123_table").is_err());
/// assert!(validate_pg_ident("table; DROP").is_err());
/// ```
pub fn validate_pg_ident(ident: &str) -> Result<(), &'static str> {
    if ident.is_empty() {
        return Err("Identifier cannot be empty");
    }
    if ident.len() > 63 {
        return Err("Identifier exceeds 63 bytes");
    }
    let mut chars = ident.chars();
    if let Some(first) = chars.next() {
        if first.is_ascii_digit() {
            return Err("Identifier cannot start with a digit");
        }
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err("Identifier must start with a letter or underscore");
        }
    }
    if chars.any(|c| !(c.is_ascii_alphanumeric() || c == '_')) {
        return Err("Identifier contains invalid characters");
    }
    Ok(())
}

/// Validate a PostgreSQL schema-qualified identifier.
///
/// Accepts formats like `"schema"."table"` or `schema.table`.
/// Each component is validated individually.
pub fn validate_qualified_ident(ident: &str) -> Result<(), &'static str> {
    let parts: Vec<&str> = ident.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err("Invalid qualified identifier format");
    }
    for part in &parts {
        let part = part.trim_matches('"');
        validate_pg_ident(part)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_idents() {
        assert!(validate_pg_ident("my_table").is_ok());
        assert!(validate_pg_ident("_private").is_ok());
        assert!(validate_pg_ident("Table123").is_ok());
        assert!(validate_pg_ident("a").is_ok());
    }

    #[test]
    fn test_invalid_idents() {
        assert!(validate_pg_ident("").is_err());
        assert!(validate_pg_ident("123_table").is_err());
        assert!(validate_pg_ident("table; DROP").is_err());
        assert!(validate_pg_ident("table-name").is_err());
        assert!(validate_pg_ident("table name").is_err());
        assert!(validate_pg_ident(
            "a_very_long_identifier_that_exceeds_sixty_three_characters_limit"
        )
        .is_err());
    }
}
