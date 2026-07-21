//! shadcn/ui Form Component Generator

use crate::generator::ir::{GeneratorEntity, GeneratorField, GeneratorFieldType, GeneratorModel};
use crate::generator::{
    GenerateError, GeneratedFile, GeneratedOutput, GenerationMetadata, Generator,
};

/// Form component generator
pub struct FormComponentGenerator;

impl FormComponentGenerator {
    /// Create a new form generator
    pub fn new() -> Self {
        Self
    }

    /// Generate form components for the model
    pub fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();
        let mut zh_dict = std::collections::HashMap::new();
        let mut en_dict = std::collections::HashMap::new();

        zh_dict.insert("form.save".to_string(), "保存".to_string());
        zh_dict.insert("form.saving".to_string(), "保存中...".to_string());
        en_dict.insert("form.save".to_string(), "Save".to_string());
        en_dict.insert("form.saving".to_string(), "Saving...".to_string());

        for entity in &model.entities {
            let form_component = self.generate_form_component(entity);
            files.push(GeneratedFile {
                path: format!("components/forms/{}-form.tsx", entity.name.kebab).into(),
                content: form_component,
                checksum: String::new(),
            });
        }

        // Output i18n skeleton dictionaries
        files.push(GeneratedFile {
            path: "locales/zh-CN.json".into(),
            content: serde_json::to_string_pretty(&zh_dict).unwrap_or_else(|_| "{}".to_string()),
            checksum: String::new(),
        });
        files.push(GeneratedFile {
            path: "locales/en.json".into(),
            content: serde_json::to_string_pretty(&en_dict).unwrap_or_else(|_| "{}".to_string()),
            checksum: String::new(),
        });

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "form_components".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate form component for an entity
    fn generate_form_component(&self, entity: &GeneratorEntity) -> String {
        let entity_name = &entity.name.pascal;
        let entity_kebab = &entity.name.kebab;

        let mut lines = vec![
            "\"use client\";".to_string(),
            "".to_string(),
            "import { Button } from \"@/components/ui/button\";".to_string(),
            "import { Input } from \"@/components/ui/input\";".to_string(),
            "import { Label } from \"@/components/ui/label\";".to_string(),
            format!("import {{ {0}Input }} from \"@/schemas/{1}.schema\";", entity_name, entity_kebab),
            format!("import {{ use{0}Form }} from \"@/hooks/use{0}Form\";", entity_name),
            "import { useT } from \"@aliothstudio/i18n\";".to_string(),
            "".to_string(),
            format!("export interface {0}FormProps {{", entity_name),
            format!("  defaultValues?: Partial<{0}Input>;", entity_name),
            "  onSubmit?: (data: any) => void | Promise<void>;".to_string(),
            "  isLoading?: boolean;".to_string(),
            "}".to_string(),
            "".to_string(),
            format!("export function {0}Form({{ defaultValues, onSubmit, isLoading }}: {0}FormProps) {{", entity_name),
            format!("  const {{ register, handleSubmit, formState: {{ errors }} }} = use{0}Form({{ defaultValues, onSubmit }});", entity_name),
            "  const t = useT();".to_string(),
            "".to_string(),
            "  return (".to_string(),
            "    <form onSubmit={handleSubmit} className=\"space-y-4\">".to_string(),
        ];

        // Add fields
        for field in &entity.fields {
            if field.name.snake == "id" {
                continue;
            }
            lines.push(self.generate_field(field));
        }

        lines.push("      <div className=\"flex justify-end gap-2 pt-4\">".to_string());
        lines.push("        <Button type=\"submit\" disabled={isLoading}>".to_string());
        lines.push("          {isLoading ? t(\"form.saving\") : t(\"form.save\")}".to_string());
        lines.push("        </Button>".to_string());
        lines.push("      </div>".to_string());
        lines.push("    </form>".to_string());
        lines.push("  );".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate a single field
    fn generate_field(&self, field: &GeneratorField) -> String {
        let name = &field.name.camel;
        let label = &field.name.pascal;

        match &field.field_type {
            GeneratorFieldType::Text => {
                format!(
                    r#"      <div className="space-y-2">
        <Label htmlFor="{0}">{1}</Label>
        <Input id="{0}" {{...register("{0}")}} placeholder="Enter {1}..." />
        {{errors.{0} && <p className="text-sm text-red-500">{{errors.{0}.message}}</p>}}
      </div>"#,
                    name, label
                )
            }
            GeneratorFieldType::Integer
            | GeneratorFieldType::BigInt
            | GeneratorFieldType::Decimal => {
                format!(
                    r#"      <div className="space-y-2">
        <Label htmlFor="{0}">{1}</Label>
        <Input id="{0}" type="number" {{...register("{0}", {{ valueAsNumber: true }})}} placeholder="Enter {1}..." />
        {{errors.{0} && <p className="text-sm text-red-500">{{errors.{0}.message}}</p>}}
      </div>"#,
                    name, label
                )
            }
            GeneratorFieldType::Boolean => {
                format!(
                    r#"      <div className="flex items-center space-x-2">
        <input type="checkbox" id="{0}" {{...register("{0}")}} className="h-4 w-4" />
        <Label htmlFor="{0}">{1}</Label>
      </div>"#,
                    name, label
                )
            }
            _ => {
                format!(
                    r#"      <div className="space-y-2">
        <Label htmlFor="{0}">{1}</Label>
        <Input id="{0}" {{...register("{0}")}} placeholder="Enter {1}..." />
        {{errors.{0} && <p className="text-sm text-red-500">{{errors.{0}.message}}</p>}}
      </div>"#,
                    name, label
                )
            }
        }
    }
}

impl Default for FormComponentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for FormComponentGenerator {
    fn name(&self) -> &'static str {
        "form_components"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        self.generate(model)
    }

    fn validate(&self, _model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["tsx"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, FieldName, PrimaryKeyType};

    fn create_test_entity() -> GeneratorEntity {
        GeneratorEntity {
            name: EntityName {
                raw: "Product".to_string(),
                snake: "product".to_string(),
                camel: "product".to_string(),
                pascal: "Product".to_string(),
                kebab: "product".to_string(),
                screaming_snake: "PRODUCT".to_string(),
                plural_snake: "products".to_string(),
                plural_pascal: "Products".to_string(),
                plural_kebab: "products".to_string(),
            },
            description: None,
            fields: vec![GeneratorField {
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
                ..Default::default()
            }],
            relations: vec![],
            annotations: vec![],
            primary_key_type: PrimaryKeyType::BigInt,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_form_component() {
        let gen = FormComponentGenerator::new();
        let entity = create_test_entity();
        let code = gen.generate_form_component(&entity);

        assert!(code.contains("function ProductForm"));
        assert!(code.contains("useProductForm"));
    }
}
