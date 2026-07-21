//! Validation Rule Conversion
//!
//! Converts IR-2 validation rules to Zod validation chains.

use crate::generator::ir::{GeneratorField, GeneratorValidation, GeneratorValidationType};

/// Validation converter
pub struct ValidationConverter;

impl ValidationConverter {
    /// Convert field validations to Zod validation chain
    pub fn convert_validations(field: &GeneratorField) -> String {
        let mut validations = Vec::new();

        for validation in &field.validations {
            let zod_validation = Self::convert_single(validation);
            if let Some(v) = zod_validation {
                validations.push(v);
            }
        }

        // Add non-empty for required string fields
        if !field.nullable && matches!(field.field_type, crate::generator::ir::GeneratorFieldType::Text) {
            // Only add nonempty if there's no min length validation
            let has_min_length = field.validations.iter().any(|v| {
                matches!(v.validation_type, GeneratorValidationType::MinLength)
            });
            if !has_min_length {
                validations.push(".min(1, { message: 'Required' })".to_string());
            }
        }

        validations.join("")
    }

    /// Convert a single validation rule to Zod chain
    fn convert_single(validation: &GeneratorValidation) -> Option<String> {
        match validation.validation_type {
            GeneratorValidationType::MinLength => {
                validation.params.get("value").map(|v| {
                    let msg = format!("Must be at least {} characters", v);
                    format!(".min({}, {{ message: '{}' }})", v, msg)
                })
            }
            GeneratorValidationType::MaxLength => {
                validation.params.get("value").map(|v| {
                    let msg = format!("Must be at most {} characters", v);
                    format!(".max({}, {{ message: '{}' }})", v, msg)
                })
            }
            GeneratorValidationType::Pattern => {
                validation.params.get("regex").map(|v| {
                    format!(".regex(/{}/, {{ message: 'Invalid format' }})", v)
                })
            }
            GeneratorValidationType::Email => {
                Some(".email({ message: 'Invalid email address' })".to_string())
            }
            GeneratorValidationType::Url => {
                Some(".url({ message: 'Invalid URL' })".to_string())
            }
            GeneratorValidationType::Min => {
                validation.params.get("value").map(|v| {
                    let msg = format!("Must be at least {}", v);
                    format!(".gte({}, {{ message: '{}' }})", v, msg)
                })
            }
            GeneratorValidationType::Max => {
                validation.params.get("value").map(|v| {
                    let msg = format!("Must be at most {}", v);
                    format!(".lte({}, {{ message: '{}' }})", v, msg)
                })
            }
            _ => None,
        }
    }

    /// Generate error message for a validation
    pub fn error_message(validation_type: GeneratorValidationType) -> String {
        match validation_type {
            GeneratorValidationType::MinLength => "Must meet minimum length requirement".to_string(),
            GeneratorValidationType::MaxLength => "Must not exceed maximum length".to_string(),
            GeneratorValidationType::Pattern => "Invalid format".to_string(),
            GeneratorValidationType::Email => "Invalid email address".to_string(),
            GeneratorValidationType::Url => "Invalid URL".to_string(),
            GeneratorValidationType::Min => "Value too small".to_string(),
            GeneratorValidationType::Max => "Value too large".to_string(),
            _ => "Invalid value".to_string(),
        }
    }
}

/// Zod validation generator for complex scenarios
pub struct ZodValidationGenerator;

impl ZodValidationGenerator {
    /// Generate a refined schema with cross-field validations
    pub fn generate_refined_schema(
        entity: &crate::generator::ir::GeneratorEntity,
    ) -> String {
        let base_schema = format!("{}Schema", entity.name.pascal);

        // Check for password confirmation fields
        let has_password = entity.fields.iter().any(|f| f.name.snake.contains("password"));
        let has_confirm = entity
            .fields
            .iter()
            .any(|f| f.name.snake.contains("confirm"));

        if has_password && has_confirm {
            format!(
                r#"export const {}RefinedSchema = {}.refine((data) => data.password === data.confirmPassword, {{
  message: "Passwords don't match",
  path: ["confirmPassword"],
}});"#,
                entity.name.pascal, base_schema
            )
        } else {
            format!("// No cross-field validations for {}", entity.name.pascal)
        }
    }

    /// Generate a super schema that combines input and output validation
    pub fn generate_super_schema(
        entity: &crate::generator::ir::GeneratorEntity,
    ) -> String {
        format!(
            r#"export const {}SuperSchema = {}Schema.merge({}InputSchema);"#,
            entity.name.pascal,
            entity.name.pascal,
            entity.name.pascal
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{FieldName, GeneratorFieldType};
    use std::collections::HashMap;

    fn create_test_field_with_validation(
        validation_type: GeneratorValidationType,
        params: HashMap<String, String>,
    ) -> GeneratorField {
        GeneratorField {
            name: FieldName {
                raw: "test".to_string(),
                snake: "test".to_string(),
                camel: "test".to_string(),
                pascal: "Test".to_string(),
            },
            field_type: GeneratorFieldType::Text,
            description: None,
            nullable: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: vec![GeneratorValidation {
                validation_type,
                params,
            }],
            annotations: vec![],
        }
    }

    #[test]
    fn test_convert_email_validation() {
        let field = create_test_field_with_validation(
            GeneratorValidationType::Email,
            HashMap::new(),
        );

        let validations = ValidationConverter::convert_validations(&field);

        assert!(validations.contains(".email"));
        assert!(validations.contains("Invalid email address"));
    }

    #[test]
    fn test_convert_min_length() {
        let mut params = HashMap::new();
        params.insert("value".to_string(), "5".to_string());

        let field = create_test_field_with_validation(GeneratorValidationType::MinLength, params);

        let validations = ValidationConverter::convert_validations(&field);

        assert!(validations.contains(".min(5"));
        assert!(validations.contains("Must be at least 5 characters"));
    }

    #[test]
    fn test_convert_max_length() {
        let mut params = HashMap::new();
        params.insert("value".to_string(), "100".to_string());

        let field = create_test_field_with_validation(GeneratorValidationType::MaxLength, params);

        let validations = ValidationConverter::convert_validations(&field);

        assert!(validations.contains(".max(100"));
        assert!(validations.contains("Must be at most 100 characters"));
    }

    #[test]
    fn test_required_field_adds_nonempty() {
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
            validations: vec![], // No min length validation
            annotations: vec![],
        };

        let validations = ValidationConverter::convert_validations(&field);

        assert!(validations.contains(".min(1"));
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(
            ValidationConverter::error_message(GeneratorValidationType::Email),
            "Invalid email address"
        );
        assert_eq!(
            ValidationConverter::error_message(GeneratorValidationType::Url),
            "Invalid URL"
        );
    }
}
