//! Quality Validation Generator Module (Phase 27)
//!
//! 提供数据质量和本体质量的自动验证功能：
//! - 完整性、准确性、一致性、时效性、有效性、唯一性验证
//! - 质量报告生成（JSON、HTML、Markdown）
//! - 本体质量分析
//! - 数据库质量检查

pub mod db_checker;
pub mod engine;
pub mod ontology_analyzer;
pub mod report;

use crate::generator::ir::quality::{OntologyQualityMetrics, QualityCheckSql, QualityReport};
use crate::generator::ir::{GeneratorEntity, GeneratorModel};
use crate::generator::{GenerateError, GeneratedOutput, GenerationMetadata, Generator};

/// Quality generator configuration
#[derive(Debug, Clone)]
pub struct QualityGeneratorConfig {
    /// Whether to generate validation functions
    pub generate_validation_fns: bool,
    /// Whether to generate SQL quality checks
    pub generate_sql_checks: bool,
    /// Whether to generate quality reports
    pub generate_reports: bool,
    /// Report formats to generate
    pub report_formats: Vec<ReportFormat>,
    /// Whether to include ontology analysis
    pub include_ontology_analysis: bool,
    /// Whether to include database quality checks
    pub include_db_checks: bool,
    /// Default sample size for database checks
    pub db_sample_size: usize,
}

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Html,
    Markdown,
}

impl Default for QualityGeneratorConfig {
    fn default() -> Self {
        Self {
            generate_validation_fns: true,
            generate_sql_checks: true,
            generate_reports: true,
            report_formats: vec![
                ReportFormat::Json,
                ReportFormat::Html,
                ReportFormat::Markdown,
            ],
            include_ontology_analysis: true,
            include_db_checks: true,
            db_sample_size: 1000,
        }
    }
}

/// Quality generator
pub struct QualityGenerator {
    config: QualityGeneratorConfig,
}

impl Default for QualityGenerator {
    fn default() -> Self {
        Self::new(QualityGeneratorConfig::default())
    }
}

impl QualityGenerator {
    /// Create new quality generator
    pub fn new(config: QualityGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate quality validation code from model
    pub fn generate(&self, model: &GeneratorModel) -> Result<Vec<GeneratedOutput>, GenerateError> {
        let mut outputs = Vec::new();

        // Generate validation functions
        if self.config.generate_validation_fns {
            let validation_output = self.generate_validation_functions(model)?;
            outputs.push(validation_output);
        }

        // Generate SQL quality checks
        if self.config.generate_sql_checks {
            let sql_output = self.generate_sql_quality_checks(model)?;
            outputs.push(sql_output);
        }

        // Generate quality reports
        if self.config.generate_reports {
            for format in &self.config.report_formats {
                let report_output = self.generate_quality_report(model, *format)?;
                outputs.push(report_output);
            }
        }

        // Generate ontology analysis
        if self.config.include_ontology_analysis {
            let ontology_output = self.generate_ontology_analysis(model)?;
            outputs.push(ontology_output);
        }

        Ok(outputs)
    }

    /// Generate validation functions for Rust
    fn generate_validation_functions(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        let mut files = Vec::new();

        for entity in &model.entities {
            if entity.quality_config.enabled {
                let content = engine::generate_rust_validation_functions(entity)?;
                let file_path = std::path::PathBuf::from(format!(
                    "src/quality/{}_quality.rs",
                    entity.name.snake
                ));

                let checksum = format!("{:x}", md5::compute(content.as_bytes()));
                files.push(crate::generator::GeneratedFile {
                    path: file_path,
                    content,
                    checksum,
                });
            }
        }

        // Generate quality module
        let mod_content = generate_quality_mod_file(&model.entities)?;
        let mod_checksum = format!("{:x}", md5::compute(mod_content.as_bytes()));
        files.push(crate::generator::GeneratedFile {
            path: std::path::PathBuf::from("src/quality/mod.rs"),
            content: mod_content,
            checksum: mod_checksum,
        });

        let c_file_count = files.len();
        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "quality_validation".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate SQL quality check queries
    fn generate_sql_quality_checks(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        let sql_checks = db_checker::generate_quality_check_sql(model);
        let content = format_sql_checks(&sql_checks);

        let files = vec![crate::generator::GeneratedFile {
            path: std::path::PathBuf::from("sql/quality_checks.sql"),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        let c_file_count = files.len();
        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "quality_sql_checks".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate quality report in specified format
    fn generate_quality_report(
        &self,
        model: &GeneratorModel,
        format: ReportFormat,
    ) -> Result<GeneratedOutput, GenerateError> {
        let metrics = ontology_analyzer::analyze_model_quality(model);

        let content = match format {
            ReportFormat::Json => report::generate_json_report(model, &metrics)?,
            ReportFormat::Html => report::generate_html_report(model, &metrics)?,
            ReportFormat::Markdown => report::generate_markdown_report(model, &metrics)?,
        };

        let extension = match format {
            ReportFormat::Json => "json",
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "md",
        };

        let files = vec![crate::generator::GeneratedFile {
            path: std::path::PathBuf::from(format!("reports/quality_report.{}", extension)),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        let c_file_count = files.len();
        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: format!("quality_report_{}", extension),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Generate ontology analysis
    fn generate_ontology_analysis(
        &self,
        model: &GeneratorModel,
    ) -> Result<GeneratedOutput, GenerateError> {
        let metrics = ontology_analyzer::analyze_model_quality(model);
        let content = ontology_analyzer::format_metrics(&metrics);

        let files = vec![crate::generator::GeneratedFile {
            path: std::path::PathBuf::from("reports/ontology_analysis.md"),
            content: content.clone(),
            checksum: format!("{:x}", md5::compute(content.as_bytes())),
        }];

        let c_file_count = files.len();
        Ok(GeneratedOutput {
            files,
            metadata: GenerationMetadata {
                generator_name: "ontology_analysis".to_string(),
                entity_count: model.entities.len(),
                c_file_count,
            },
        })
    }

    /// Run quality validation on entity
    pub fn validate_entity(&self, entity: &GeneratorEntity) -> QualityReport {
        engine::validate_entity_quality(entity)
    }

    /// Analyze model ontology quality
    pub fn analyze_ontology(&self, model: &GeneratorModel) -> OntologyQualityMetrics {
        ontology_analyzer::analyze_model_quality(model)
    }
}

impl Generator for QualityGenerator {
    fn name(&self) -> &'static str {
        "quality"
    }

    fn generate(&self, model: &GeneratorModel) -> Result<GeneratedOutput, GenerateError> {
        let outputs = self.generate(model)?;

        // Flatten outputs
        let mut all_files = Vec::new();
        let mut entity_count = 0;

        for output in outputs {
            entity_count += output.metadata.entity_count;
            all_files.extend(output.files);
        }

        let c_file_count = all_files.len();

        Ok(GeneratedOutput {
            files: all_files,
            metadata: GenerationMetadata {
                generator_name: self.name().to_string(),
                entity_count,
                c_file_count,
            },
        })
    }

    fn validate(&self, model: &GeneratorModel) -> Result<(), crate::generator::ValidationError> {
        // Validate quality rules are well-formed
        for entity in &model.entities {
            for rule in &entity.quality_rules {
                if rule.threshold < 0.0 || rule.threshold > 1.0 {
                    return Err(crate::generator::ValidationError::InvalidField {
                        entity: entity.name.raw.clone(),
                        field: rule.field_name.clone().unwrap_or_default(),
                        reason: format!(
                            "Quality threshold must be between 0.0 and 1.0, got {}",
                            rule.threshold
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    fn supports_incremental(&self) -> bool {
        true
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["rs", "sql", "json", "html", "md"]
    }
}

/// Format SQL checks for output
fn format_sql_checks(checks: &[QualityCheckSql]) -> String {
    let mut output = String::new();
    output.push_str("-- Auto-generated quality check SQL queries\n");
    output.push_str("-- Generated by AliothStudio Quality Generator\n\n");

    for check in checks {
        output.push_str(&format!("-- Check: {} ({:?})\n", check.name, check.metric));
        output.push_str(&format!("-- Threshold: {:.2}%\n", check.threshold * 100.0));
        output.push_str(&check.sql);
        output.push_str(";\n\n");
    }

    output
}

/// Generate quality module file
fn generate_quality_mod_file(entities: &[GeneratorEntity]) -> Result<String, GenerateError> {
    let mut content = String::new();
    content.push_str("//! Quality Validation Module\n\n");

    // Add module declarations
    for entity in entities {
        if entity.quality_config.enabled {
            content.push_str(&format!("pub mod {}_quality;\n", entity.name.snake));
        }
    }

    content.push_str("\nuse serde::{Deserialize, Serialize};\n");
    content.push_str("use std::collections::HashMap;\n\n");

    // Re-export validation functions
    content.push_str("/// Run all quality validations\n");
    content.push_str("pub async fn run_all_validations() -> QualityResult {\n");
    content.push_str("    let mut results = QualityResult::default();\n\n");

    for entity in entities {
        if entity.quality_config.enabled {
            content.push_str(&format!(
                "    results.extend(validate_{}_quality().await);\n",
                entity.name.snake
            ));
        }
    }

    content.push_str("\n    results\n");
    content.push_str("}\n\n");

    // Add result types
    content.push_str("#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n");
    content.push_str("pub struct QualityResult {\n");
    content.push_str("    pub entity_results: HashMap<String, EntityQualityResult>,\n");
    content.push_str("}\n\n");

    content.push_str("impl QualityResult {\n");
    content.push_str("    pub fn extend(&mut self, result: EntityQualityResult) {\n");
    content.push_str("        self.entity_results.insert(result.entity_name.clone(), result);\n");
    content.push_str("    }\n");
    content.push_str("}\n\n");

    content.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    content.push_str("pub struct EntityQualityResult {\n");
    content.push_str("    pub entity_name: String,\n");
    content.push_str("    pub overall_score: f64,\n");
    content.push_str("    pub passed: bool,\n");
    content.push_str("}\n");

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::quality::{QualityMetric, QualityResultType};

    #[test]
    fn test_quality_generator_default() {
        let generator = QualityGenerator::default();
        assert!(generator.config.generate_validation_fns);
        assert!(generator.config.generate_reports);
        assert_eq!(generator.config.report_formats.len(), 3);
    }

    #[test]
    fn test_quality_generator_name() {
        let generator = QualityGenerator::default();
        assert_eq!(generator.name(), "quality");
    }

    #[test]
    fn test_format_sql_checks() {
        let checks = vec![QualityCheckSql {
            name: "test_check".to_string(),
            metric: QualityMetric::Completeness,
            sql: "SELECT COUNT(*) FROM users WHERE email IS NULL".to_string(),
            result_type: QualityResultType::Count,
            threshold: 0.95,
        }];

        let output = format_sql_checks(&checks);
        assert!(output.contains("test_check"));
        assert!(output.contains("Completeness"));
        assert!(output.contains("SELECT COUNT(*)"));
    }
}
