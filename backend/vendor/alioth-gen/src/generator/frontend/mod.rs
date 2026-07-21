//! Frontend Component Generator (Phase 6)
//!
//! Generates shadcn/ui form components, data tables, and analytics charts.
//!
//! ## Features
//!
//! - **Form Components**: shadcn/ui forms with validation
//! - **Data Tables**: Full CRUD data tables with pagination/sorting/filtering
//! - **Charts**: Analytics dashboards with recharts
//! - **Dialogs**: Create/Edit dialogs
//! - **Pages**: Next.js page components
//!
//! ## Generated Files
//!
//! ```text
//! components/
//! ├── forms/
//! │   ├── {entity}-form.tsx       # Form component
//! │   └── index.ts
//! ├── dialogs/
//! │   ├── create-{entity}-dialog.tsx
//! │   └── edit-{entity}-dialog.tsx
//! ├── tables/
//! │   ├── {entity}-table.tsx      # Data table
//! │   ├── {entity}-columns.tsx    # Column definitions
//! │   └── index.ts
//! └── analytics/
//!     ├── {entity}-analytics.tsx  # Analytics dashboard
//!     └── {entity}-stats.tsx      # Stat cards
//!
//! app/{entity_plural}/
//! └── page.tsx                     # List page
//! ```

mod charts;
mod form;
mod table;

pub use charts::{ChartComponentGenerator, ChartGeneratorOptions, ChartType};
pub use form::FormComponentGenerator;
pub use table::DataTableGenerator;

use crate::generator::ir::GeneratorModel;
use crate::generator::{GenerateError, GeneratedOutput, Generator};

/// Combined frontend component generator
pub struct FrontendComponentGenerator {
    form_gen: FormComponentGenerator,
    table_gen: DataTableGenerator,
    chart_gen: ChartComponentGenerator,
}

impl FrontendComponentGenerator {
    /// Create a new frontend component generator
    pub fn new() -> Self {
        Self {
            form_gen: FormComponentGenerator::new(),
            table_gen: DataTableGenerator::new(),
            chart_gen: ChartComponentGenerator::new(),
        }
    }

    /// Configure form generator
    pub fn with_form_gen(mut self, gen: FormComponentGenerator) -> Self {
        self.form_gen = gen;
        self
    }

    /// Configure table generator
    pub fn with_table_gen(mut self, gen: DataTableGenerator) -> Self {
        self.table_gen = gen;
        self
    }

    /// Configure chart generator
    pub fn with_chart_gen(mut self, gen: ChartComponentGenerator) -> Self {
        self.chart_gen = gen;
        self
    }

    /// Generate all frontend components
    pub fn generate_all(
        &self,
        model: &GeneratorModel,
    ) -> Result<FrontendGeneratedOutput, GenerateError> {
        let forms = self.form_gen.generate(model)?;
        let tables = self.table_gen.generate(model)?;
        let charts = self.chart_gen.generate(model)?;

        Ok(FrontendGeneratedOutput {
            forms,
            tables,
            charts,
        })
    }
}

impl Default for FrontendComponentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for FrontendComponentGenerator {
    fn name(&self) -> &'static str {
        "frontend_components"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let all = self.generate_all(model)?;

        // Combine all files
        let mut files = Vec::new();
        files.extend(all.forms.files);
        files.extend(all.tables.files);
        files.extend(all.charts.files);

        let c_file_count = files.len();

        Ok(GeneratedOutput {
            files,
            metadata: crate::generator::GenerationMetadata {
                generator_name: self.name().to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
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

/// Output from frontend component generation
pub struct FrontendGeneratedOutput {
    /// Form components
    pub forms: GeneratedOutput,
    /// Table components
    pub tables: GeneratedOutput,
    /// Chart components
    pub charts: GeneratedOutput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::{EntityName, GeneratorEntity, ModelMetadata, PrimaryKeyType};

    fn create_test_model() -> GeneratorModel {
        GeneratorModel {
            i18n_config: None,
            entities: vec![GeneratorEntity {
                name: EntityName {
                    raw: "Task".to_string(),
                    snake: "task".to_string(),
                    camel: "task".to_string(),
                    pascal: "Task".to_string(),
                    kebab: "task".to_string(),
                    screaming_snake: "TASK".to_string(),
                    plural_snake: "tasks".to_string(),
                    plural_pascal: "Tasks".to_string(),
                    plural_kebab: "tasks".to_string(),
                },
                description: Some("Task entity".to_string()),
                fields: vec![],
                relations: vec![],
                annotations: vec![],
                primary_key_type: PrimaryKeyType::BigInt,
                ..Default::default()
            }],
            enums: vec![],
            metadata: ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_frontend_component_generator() {
        let model = create_test_model();
        let gen = FrontendComponentGenerator::new();
        let output = gen.generate_all(&model).unwrap();

        assert_eq!(output.forms.metadata.entity_count, 1);
        assert_eq!(output.tables.metadata.entity_count, 1);
        assert_eq!(output.charts.metadata.entity_count, 1);
    }

    #[test]
    fn test_combined_generator() {
        let model = create_test_model();
        let gen = FrontendComponentGenerator::new();
        let output = gen.generate(&model).unwrap();

        // Should have: form files + table files + chart files
        assert!(output.files.len() >= 6);
    }
}
