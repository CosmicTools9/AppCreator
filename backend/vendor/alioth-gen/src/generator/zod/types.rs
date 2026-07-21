//! Zod Type Mapping
//!
//! Maps IR-2 field types to Zod type strings.

use crate::generator::ir::GeneratorFieldType;

/// Zod type configuration
#[derive(Debug, Clone)]
pub struct ZodTypeConfig {
    /// Use strict mode (no coercions)
    pub strict: bool,
    /// Generate branded types for IDs
    pub branded_ids: bool,
    /// Use preprocess for date parsing
    pub preprocess_dates: bool,
    /// Coerce numbers from strings
    pub coerce_numbers: bool,
}

impl Default for ZodTypeConfig {
    fn default() -> Self {
        Self {
            strict: false,
            branded_ids: true,
            preprocess_dates: true,
            coerce_numbers: true,
        }
    }
}

impl ZodTypeConfig {
    /// Create strict config
    pub fn strict() -> Self {
        Self {
            strict: true,
            branded_ids: true,
            preprocess_dates: false,
            coerce_numbers: false,
        }
    }
}

/// Zod type mapper
#[derive(Debug, Clone)]
pub struct ZodTypeMapper {
    config: ZodTypeConfig,
}

impl ZodTypeMapper {
    /// Create a new type mapper with default config
    pub fn new() -> Self {
        Self {
            config: ZodTypeConfig::default(),
        }
    }

    /// Create a type mapper with custom config
    pub fn with_config(config: ZodTypeConfig) -> Self {
        Self { config }
    }

    /// Map IR-2 field type to Zod type string
    pub fn map_field_type(&self, field_type: &GeneratorFieldType) -> String {
        match field_type {
            GeneratorFieldType::Text => "z.string()".to_string(),
            GeneratorFieldType::Integer => {
                if self.config.coerce_numbers {
                    "z.coerce.number().int()".to_string()
                } else {
                    "z.number().int()".to_string()
                }
            }
            GeneratorFieldType::BigInt => {
                if self.config.branded_ids {
                    "z.bigint()".to_string()
                } else {
                    "z.bigint()".to_string()
                }
            }
            GeneratorFieldType::Decimal => {
                if self.config.coerce_numbers {
                    "z.coerce.number()".to_string()
                } else {
                    "z.number()".to_string()
                }
            }
            GeneratorFieldType::Boolean => {
                if self.config.strict {
                    "z.boolean()".to_string()
                } else {
                    "z.coerce.boolean()".to_string()
                }
            }
            GeneratorFieldType::DateTime => {
                if self.config.preprocess_dates {
                    "z.preprocess((val) => (typeof val === 'string' || typeof val === 'number' ? new Date(val) : val), z.date())".to_string()
                } else {
                    "z.date()".to_string()
                }
            }
            GeneratorFieldType::Uuid => "z.string().uuid()".to_string(),
            GeneratorFieldType::Json => "z.record(z.any())".to_string(),
            GeneratorFieldType::Enum(name) => {
                format!("z.enum({}Values)", name)
            }
            GeneratorFieldType::Reference(target) => {
                if self.config.branded_ids {
                    format!("z.bigint().brand<'{}Id'>()", target)
                } else {
                    "z.bigint()".to_string()
                }
            }
        }
    }

    /// Map to optional Zod type
    pub fn map_optional(&self, field_type: &GeneratorFieldType) -> String {
        format!("{}.optional()", self.map_field_type(field_type))
    }

    /// Map to nullable Zod type
    pub fn map_nullable(&self, field_type: &GeneratorFieldType) -> String {
        format!("{}.nullable()", self.map_field_type(field_type))
    }

    /// Map to array Zod type
    pub fn map_array(&self, field_type: &GeneratorFieldType) -> String {
        format!("{}.array()", self.map_field_type(field_type))
    }

    /// Check if a field type can be mapped to Zod
    pub fn can_map(field_type: &GeneratorFieldType) -> bool {
        matches!(
            field_type,
            GeneratorFieldType::Text
                | GeneratorFieldType::Integer
                | GeneratorFieldType::BigInt
                | GeneratorFieldType::Decimal
                | GeneratorFieldType::Boolean
                | GeneratorFieldType::DateTime
                | GeneratorFieldType::Uuid
                | GeneratorFieldType::Json
                | GeneratorFieldType::Enum(_)
                | GeneratorFieldType::Reference(_)
        )
    }

    /// Get the default value expression for a type
    pub fn default_value_expr(&self, field_type: &GeneratorFieldType, value: &str) -> String {
        match field_type {
            GeneratorFieldType::Text => format!("'{}'", value.replace('\'', "\\'")),
            GeneratorFieldType::Integer | GeneratorFieldType::BigInt | GeneratorFieldType::Decimal => {
                value.to_string()
            }
            GeneratorFieldType::Boolean => value.to_lowercase(),
            GeneratorFieldType::DateTime => {
                if value.eq_ignore_ascii_case("now") {
                    "new Date()".to_string()
                } else {
                    format!("new Date('{}')", value)
                }
            }
            _ => "undefined".to_string(),
        }
    }
}

impl Default for ZodTypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_basic_types() {
        let mapper = ZodTypeMapper::new();

        assert_eq!(mapper.map_field_type(&GeneratorFieldType::Text), "z.string()");
        assert_eq!(
            mapper.map_field_type(&GeneratorFieldType::Integer),
            "z.coerce.number().int()"
        );
        assert_eq!(mapper.map_field_type(&GeneratorFieldType::Boolean), "z.coerce.boolean()");
    }

    #[test]
    fn test_map_uuid() {
        let mapper = ZodTypeMapper::new();
        assert_eq!(
            mapper.map_field_type(&GeneratorFieldType::Uuid),
            "z.string().uuid()"
        );
    }

    #[test]
    fn test_map_optional() {
        let mapper = ZodTypeMapper::new();
        assert_eq!(
            mapper.map_optional(&GeneratorFieldType::Text),
            "z.string().optional()"
        );
    }

    #[test]
    fn test_map_nullable() {
        let mapper = ZodTypeMapper::new();
        assert_eq!(
            mapper.map_nullable(&GeneratorFieldType::Text),
            "z.string().nullable()"
        );
    }

    #[test]
    fn test_map_array() {
        let mapper = ZodTypeMapper::new();
        assert_eq!(
            mapper.map_array(&GeneratorFieldType::Text),
            "z.string().array()"
        );
    }

    #[test]
    fn test_strict_mode() {
        let config = ZodTypeConfig::strict();
        let mapper = ZodTypeMapper::with_config(config);

        assert_eq!(mapper.map_field_type(&GeneratorFieldType::Boolean), "z.boolean()");
        assert_eq!(
            mapper.map_field_type(&GeneratorFieldType::Integer),
            "z.number().int()"
        );
    }

    #[test]
    fn test_can_map() {
        assert!(ZodTypeMapper::can_map(&GeneratorFieldType::Text));
        assert!(ZodTypeMapper::can_map(&GeneratorFieldType::Integer));
        assert!(ZodTypeMapper::can_map(&GeneratorFieldType::Uuid));
    }

    #[test]
    fn test_default_value_expr() {
        let mapper = ZodTypeMapper::new();

        assert_eq!(
            mapper.default_value_expr(&GeneratorFieldType::Text, "hello"),
            "'hello'"
        );
        assert_eq!(
            mapper.default_value_expr(&GeneratorFieldType::Integer, "42"),
            "42"
        );
        assert_eq!(
            mapper.default_value_expr(&GeneratorFieldType::Boolean, "true"),
            "true"
        );
    }
}
