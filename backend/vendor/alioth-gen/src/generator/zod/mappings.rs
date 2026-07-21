//! Zod Type Mappings
//!
//! Maps IR-2 types to Zod types and validation chains.

use crate::generator::ir::{
    GeneratorField, GeneratorFieldType, GeneratorValidation, GeneratorValidationType,
};

/// Zod type mapping configuration
#[derive(Debug, Clone)]
pub struct ZodMappingConfig {
    /// Enable strict mode (fail on unknown keys)
    pub strict: bool,
    /// Enable type coercion for primitives
    pub coerce: bool,
    /// Custom error message locale
    pub locale: String,
}

impl Default for ZodMappingConfig {
    fn default() -> Self {
        Self {
            strict: true,
            coerce: true,
            locale: "zh-CN".to_string(),
        }
    }
}

/// Type mapper for Zod
pub struct ZodTypeMapper {
    config: ZodMappingConfig,
}

impl ZodTypeMapper {
    /// Create a new type mapper with default config
    pub fn new() -> Self {
        Self {
            config: ZodMappingConfig::default(),
        }
    }

    /// Create a type mapper with custom config
    pub fn with_config(config: ZodMappingConfig) -> Self {
        Self { config }
    }

    /// Map an IR-2 field to a Zod type string
    pub fn map_field(&self, field: &GeneratorField) -> String {
        let base_type = self.map_type(&field.field_type);
        let mut validations = self.build_validations(&field.validations, &field.field_type);

        // Add cardinality constraints from OWL constraints
        let cardinality_validations = self.build_cardinality_validations(field);
        validations.extend(cardinality_validations);

        let chain = if validations.is_empty() {
            base_type
        } else {
            format!("{}.{}", base_type, validations.join(")."))
        };

        if field.nullable {
            format!("{}.nullish()", chain)
        } else {
            chain
        }
    }

    /// Build cardinality validations from OWL constraints
    fn build_cardinality_validations(&self, field: &GeneratorField) -> Vec<String> {
        let mut chain = Vec::new();

        // Handle array types with cardinality constraints
        if let (Some(min), Some(max)) = (field.min_cardinality, field.max_cardinality) {
            if min == max {
                chain.push(format!("length({})", min));
            } else {
                chain.push(format!("min({})", min));
                chain.push(format!("max({})", max));
            }
        } else if let Some(min) = field.min_cardinality {
            chain.push(format!("min({})", min));
        } else if let Some(max) = field.max_cardinality {
            chain.push(format!("max({})", max));
        }

        // functional property implies max(1)
        if field.is_functional && field.max_cardinality.is_none() {
            chain.push("max(1)".to_string());
        }

        chain
    }

    /// Map IR-2 type to base Zod type
    fn map_type(&self, field_type: &GeneratorFieldType) -> String {
        let coerce = if self.config.coerce { "coerce." } else { "" };

        match field_type {
            GeneratorFieldType::Text => "z.string()".to_string(),
            GeneratorFieldType::Integer => {
                if self.config.coerce {
                    format!("z.{}number()", coerce)
                } else {
                    "z.number()".to_string()
                }
            }
            GeneratorFieldType::BigInt => {
                if self.config.coerce {
                    format!("z.{}bigint()", coerce)
                } else {
                    "z.bigint()".to_string()
                }
            }
            GeneratorFieldType::Decimal => {
                if self.config.coerce {
                    format!("z.{}number()", coerce)
                } else {
                    "z.number()".to_string()
                }
            }
            GeneratorFieldType::Boolean => {
                if self.config.coerce {
                    format!("z.{}boolean()", coerce)
                } else {
                    "z.boolean()".to_string()
                }
            }
            GeneratorFieldType::DateTime => "z.date()".to_string(),
            GeneratorFieldType::Uuid => "z.string().uuid()".to_string(),
            GeneratorFieldType::Json => "z.record(z.any())".to_string(),
            GeneratorFieldType::Enum(name) => format!("z.nativeEnum({})", name),
            GeneratorFieldType::Reference(target) => {
                format!("z.bigint() // Reference to {}", target)
            }
        }
    }

    /// Build validation chain from IR-2 validations
    fn build_validations(
        &self,
        validations: &[GeneratorValidation],
        _field_type: &GeneratorFieldType,
    ) -> Vec<String> {
        let mut chain = Vec::new();

        for validation in validations {
            let validation_str = match &validation.validation_type {
                GeneratorValidationType::MinLength => {
                    if let Some(min) = validation.params.get("min") {
                        format!("min({})", min)
                    } else {
                        continue;
                    }
                }
                GeneratorValidationType::MaxLength => {
                    if let Some(max) = validation.params.get("max") {
                        format!("max({})", max)
                    } else {
                        continue;
                    }
                }
                GeneratorValidationType::Pattern => {
                    if let Some(pattern) = validation.params.get("pattern") {
                        format!("regex(/^{}$/)", pattern)
                    } else {
                        continue;
                    }
                }
                GeneratorValidationType::Min => {
                    if let Some(min) = validation.params.get("min") {
                        format!("gte({})", min)
                    } else {
                        continue;
                    }
                }
                GeneratorValidationType::Max => {
                    if let Some(max) = validation.params.get("max") {
                        format!("lte({})", max)
                    } else {
                        continue;
                    }
                }
                GeneratorValidationType::Email => "email()".to_string(),
                GeneratorValidationType::Url => "url()".to_string(),
                GeneratorValidationType::Custom(_) => continue, // Skip custom validations
            };

            chain.push(validation_str);
        }

        chain
    }

    /// Get default value for a type
    pub fn default_value_for_type(&self, field_type: &GeneratorFieldType) -> Option<String> {
        match field_type {
            GeneratorFieldType::Text => Some("\"\"".to_string()),
            GeneratorFieldType::Integer => Some("0".to_string()),
            GeneratorFieldType::BigInt => Some("0n".to_string()),
            GeneratorFieldType::Decimal => Some("0".to_string()),
            GeneratorFieldType::Boolean => Some("false".to_string()),
            GeneratorFieldType::DateTime => Some("new Date()".to_string()),
            GeneratorFieldType::Uuid => None, // No sensible default
            GeneratorFieldType::Json => Some("{}".to_string()),
            GeneratorFieldType::Enum(_) => None,
            GeneratorFieldType::Reference(_) => Some("0n".to_string()),
        }
    }

    /// Generate form default values object
    pub fn generate_default_values(&self, fields: &[GeneratorField]) -> Vec<(String, String)> {
        fields
            .iter()
            .filter(|f| f.name.snake != "id") // Skip id field
            .filter_map(|f| {
                self.default_value_for_type(&f.field_type)
                    .map(|val| (f.name.camel.clone(), val))
            })
            .collect()
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
    use crate::generator::ir::{FieldName, GeneratorValidation};

    #[test]
    fn test_map_text_type() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "name".to_string(),
                snake: "name".to_string(),
                camel: "name".to_string(),
                pascal: "Name".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        assert_eq!(mapper.map_field(&field), "z.string()");
    }

    #[test]
    fn test_map_integer_with_coerce() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "age".to_string(),
                snake: "age".to_string(),
                camel: "age".to_string(),
                pascal: "Age".to_string(),
            },
            field_type: GeneratorFieldType::Integer,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        assert_eq!(mapper.map_field(&field), "z.coerce.number()");
    }

    #[test]
    fn test_map_nullable() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "age".to_string(),
                snake: "age".to_string(),
                camel: "age".to_string(),
                pascal: "Age".to_string(),
            },
            field_type: GeneratorFieldType::Integer,
            description: None,
            nullable: true,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        assert_eq!(mapper.map_field(&field), "z.coerce.number().nullish()");
    }

    #[test]
    fn test_map_with_validations() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "email".to_string(),
                snake: "email".to_string(),
                camel: "email".to_string(),
                pascal: "Email".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![
                GeneratorValidation {
                    validation_type: GeneratorValidationType::MinLength,
                    params: [("min".to_string(), "5".to_string())].into_iter().collect(),
                },
                GeneratorValidation {
                    validation_type: GeneratorValidationType::MaxLength,
                    params: [("max".to_string(), "100".to_string())]
                        .into_iter()
                        .collect(),
                },
            ],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        let result = mapper.map_field(&field);
        assert!(result.contains("min(5)"));
        assert!(result.contains("max(100)"));
    }

    #[test]
    fn test_map_with_cardinality_constraints() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "tags".to_string(),
                snake: "tags".to_string(),
                camel: "tags".to_string(),
                pascal: "Tags".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: Some(1),
            max_cardinality: Some(5),
            is_functional: false,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        let result = mapper.map_field(&field);
        assert!(result.contains("min(1)"));
        assert!(result.contains("max(5)"));
    }

    #[test]
    fn test_map_functional_property() {
        let mapper = ZodTypeMapper::new();
        let field = GeneratorField {
            name: FieldName {
                raw: "manager".to_string(),
                snake: "manager".to_string(),
                camel: "manager".to_string(),
                pascal: "Manager".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![],
            annotations: vec![],
            domain: None,
            range: None,
            min_cardinality: None,
            max_cardinality: None,
            is_functional: true,
            constraints: vec![],
            throws_clauses: vec![],
            quality_rules: vec![],
        };

        let result = mapper.map_field(&field);
        assert!(result.contains("max(1)"));
    }
}
