//! CLI Integration Tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_preview_command_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test GeneratorModel JSON file
        let model = crate::generator::ir::GeneratorModel {
            i18n_config: None,
            entities: vec![crate::generator::ir::GeneratorEntity {
                name: crate::generator::ir::EntityName {
                    raw: "User".to_string(),
                    snake: "user".to_string(),
                    camel: "user".to_string(),
                    pascal: "User".to_string(),
                    kebab: "user".to_string(),
                    screaming_snake: "USER".to_string(),
                    plural_snake: "users".to_string(),
                    plural_pascal: "Users".to_string(),
                    plural_kebab: "users".to_string(),
                },
                description: None,
                fields: vec![crate::generator::ir::GeneratorField {
                    name: crate::generator::ir::FieldName {
                        raw: "name".to_string(),
                        snake: "name".to_string(),
                        camel: "name".to_string(),
                        pascal: "Name".to_string(),
                    },
                    field_type: crate::generator::ir::GeneratorFieldType::Text,
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
                }],
                relations: vec![],
                annotations: vec![],
                primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
                parent_classes: vec![],
                equivalent_classes: vec![],
                disjoint_classes: vec![],
                is_abstract: false,
                inheritance_depth: 0,
                state_machine: Default::default(),
                transitions: vec![],
                lifecycle_hooks: vec![],
                business_rules: vec![],
                swrl_rules: vec![],
                constraints: vec![],
                quality_rules: vec![],
                quality_config: Default::default(),
                parent_tables: vec![],
            }],
            enums: vec![],
            metadata: crate::generator::ir::ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        };
        let json_path = temp_dir.path().join("test.model.json");
        fs::write(&json_path, serde_json::to_string_pretty(&model).unwrap()).unwrap();

        // Create args for preview
        let args = PreviewArgs {
            input: json_path.to_string_lossy().to_string(),
            output: temp_dir.path().join("generated"),
            diff: false,
            save: None,
            include: None,
            exclude: None,
            generators: Some("zod".to_string()),
            check_conflicts: true,
        };

        // Run preview (this should succeed)
        let result = CliRunner::run_preview(args);
        assert!(result.is_ok(), "Preview command should succeed");
    }

    #[test]
    fn test_preview_command_with_save() {
        let temp_dir = TempDir::new().unwrap();
        let save_dir = temp_dir.path().join("preview_output");

        // Create a test GeneratorModel JSON file
        let model = crate::generator::ir::GeneratorModel {
            i18n_config: None,
            entities: vec![crate::generator::ir::GeneratorEntity {
                name: crate::generator::ir::EntityName {
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
                fields: vec![crate::generator::ir::GeneratorField {
                    name: crate::generator::ir::FieldName {
                        raw: "sku".to_string(),
                        snake: "sku".to_string(),
                        camel: "sku".to_string(),
                        pascal: "Sku".to_string(),
                    },
                    field_type: crate::generator::ir::GeneratorFieldType::Text,
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
                }],
                relations: vec![],
                annotations: vec![],
                primary_key_type: crate::generator::ir::PrimaryKeyType::BigInt,
                parent_classes: vec![],
                equivalent_classes: vec![],
                disjoint_classes: vec![],
                is_abstract: false,
                inheritance_depth: 0,
                state_machine: Default::default(),
                transitions: vec![],
                lifecycle_hooks: vec![],
                business_rules: vec![],
                swrl_rules: vec![],
                constraints: vec![],
                quality_rules: vec![],
                quality_config: Default::default(),
                parent_tables: vec![],
            }],
            enums: vec![],
            metadata: crate::generator::ir::ModelMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator_version: "test".to_string(),
            },
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        };
        let json_path = temp_dir.path().join("product.model.json");
        fs::write(&json_path, serde_json::to_string_pretty(&model).unwrap()).unwrap();

        let args = PreviewArgs {
            input: json_path.to_string_lossy().to_string(),
            output: temp_dir.path().join("generated"),
            diff: false,
            save: Some(save_dir.clone()),
            include: None,
            exclude: None,
            generators: Some("zod".to_string()),
            check_conflicts: true,
        };

        let result = CliRunner::run_preview(args);
        assert!(result.is_ok());

        // Check that preview summary was saved
        let summary_path = save_dir.join("preview-summary.json");
        assert!(summary_path.exists(), "Preview summary should be saved");
    }

    #[test]
    fn test_batch_args_parsing() {
        let args = BatchArgs {
            models: "1,2,3,4,5".to_string(),
            output: PathBuf::from("./out"),
            generators: Some("zod".to_string()),
            parallel: false,
            max_concurrent: 4,
            continue_on_error: true,
            dry_run: false,
        };

        assert_eq!(args.model_ids(), vec![1, 2, 3, 4, 5]);
        assert_eq!(args.generators().unwrap(), vec!["zod"]);
    }

    #[test]
    fn test_batch_args_empty_models() {
        let args = BatchArgs {
            models: "".to_string(),
            output: PathBuf::from("./out"),
            generators: None,
            parallel: false,
            max_concurrent: 4,
            continue_on_error: false,
            dry_run: false,
        };

        assert!(args.model_ids().is_empty());
    }

    #[test]
    fn test_batch_args_invalid_ids() {
        let args = BatchArgs {
            models: "1,invalid,3,not_a_number".to_string(),
            output: PathBuf::from("./out"),
            generators: None,
            parallel: false,
            max_concurrent: 4,
            continue_on_error: false,
            dry_run: false,
        };

        // Only valid IDs should be parsed
        assert_eq!(args.model_ids(), vec![1, 3]);
    }

    #[test]
    fn test_history_args_defaults() {
        let args = HistoryArgs {
            model: Some(123),
            limit: 10,
            offset: 0,
            generator: None,
            format: HistoryOutputFormat::Table,
            detailed: false,
            fk_history: None,
        };

        assert_eq!(args.model, Some(123));
        assert_eq!(args.limit, 10);
        assert_eq!(args.offset, 0);
    }

    #[test]
    fn test_rollback_args() {
        let args = RollbackArgs {
            history: 456,
            output: PathBuf::from("./out"),
            force: false,
            preview: true,
            skip_protected_check: false,
        };

        assert_eq!(args.history, 456);
        assert!(!args.force);
        assert!(args.preview);
        assert!(!args.skip_protected_check);
    }

    #[test]
    fn test_glob_match_basic() {
        assert!(CliRunner::glob_match("*.rs", "test.rs"));
        assert!(CliRunner::glob_match("*.rs", "path/to/test.rs"));
        assert!(!CliRunner::glob_match("*.rs", "test.txt"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(CliRunner::glob_match("file?.txt", "file1.txt"));
        assert!(CliRunner::glob_match("file?.txt", "fileA.txt"));
        assert!(!CliRunner::glob_match("file?.txt", "file12.txt"));
    }

    #[test]
    fn test_glob_match_star() {
        assert!(CliRunner::glob_match("*test*", "this_is_a_test_file"));
        assert!(CliRunner::glob_match("*", "anything"));
        assert!(CliRunner::glob_match("a*b", "ab"));
        assert!(CliRunner::glob_match("a*b", "a123b"));
    }

    #[test]
    fn test_glob_match_complex() {
        // Note: The glob_match function implements simple pattern matching
        // The ** pattern would require more sophisticated handling
        // For now, we test the basic patterns that are supported
        assert!(CliRunner::glob_match("src/*.rs", "src/lib.rs"));
        assert!(CliRunner::glob_match("src/*/*.rs", "src/module/test.rs"));
        assert!(!CliRunner::glob_match("src/*.rs", "tests/test.rs"));
    }

    #[test]
    fn test_glob_match_multiple_patterns() {
        let patterns = ["*.rs", "*.ts"];
        let file = "test.rs";

        let matches = patterns.iter().any(|p| CliRunner::glob_match(p, file));
        assert!(matches);

        let file = "test.ts";
        let matches = patterns.iter().any(|p| CliRunner::glob_match(p, file));
        assert!(matches);

        let file = "test.js";
        let matches = patterns.iter().any(|p| CliRunner::glob_match(p, file));
        assert!(!matches);
    }

    #[test]
    fn test_preview_args_pattern_filtering() {
        let args = PreviewArgs {
            input: "test.dsl".to_string(),
            output: PathBuf::from("./out"),
            diff: false,
            save: None,
            include: Some("*.rs,*.ts".to_string()),
            exclude: Some("*.test.rs".to_string()),
            generators: None,
            check_conflicts: true,
        };

        let include = args.include_patterns();
        assert_eq!(include, vec!["*.rs", "*.ts"]);

        let exclude = args.exclude_patterns();
        assert_eq!(exclude, vec!["*.test.rs"]);
    }

    #[test]
    fn test_generate_args_with_dry_run() {
        let args = GenerateArgs {
            input: Some(PathBuf::from("test.dsl")),
            output: PathBuf::from("./out"),
            app_name: None,
            config: None,
            generators: Some("zod".to_string()),
            watch: false,
            dry_run: true,
            fk_model: Some(123),
            from_ontology: None,
        };

        assert!(args.dry_run);
        assert_eq!(args.fk_model, Some(123));
    }

    #[test]
    fn test_export_format_auto_detection() {
        // Test that ExportFormat has the expected variants
        let formats = [ExportFormat::Auto, ExportFormat::Dsl, ExportFormat::Json];

        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_history_output_formats() {
        let formats = [
            HistoryOutputFormat::Table,
            HistoryOutputFormat::Json,
            HistoryOutputFormat::Yaml,
        ];

        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_cli_error_types() {
        let io_err = CliError::Io("test io".to_string());
        assert!(io_err.to_string().contains("IO Error"));

        let parse_err = CliError::Parse("test parse".to_string());
        assert!(parse_err.to_string().contains("Parse Error"));

        let gen_err = CliError::Generate("test gen".to_string());
        assert!(gen_err.to_string().contains("Generate Error"));

        let config_err = CliError::Config("test config".to_string());
        assert!(config_err.to_string().contains("Config Error"));

        let invalid_args = CliError::InvalidArgs("test args".to_string());
        assert!(invalid_args.to_string().contains("Invalid Arguments"));

        let batch_failed = CliError::BatchFailed("test batch".to_string());
        assert!(batch_failed.to_string().contains("Batch Failed"));

        let registry_err = CliError::Registry("test registry".to_string());
        assert!(registry_err.to_string().contains("Registry Error"));
    }

    #[test]
    fn test_cli_error_is_error_trait() {
        fn assert_error<E: std::error::Error>(_e: E) {}

        assert_error(CliError::Io("test".to_string()));
        assert_error(CliError::Parse("test".to_string()));
    }

    #[test]
    fn test_version_validation_valid() {
        // Valid semantic versions
        assert!(CliRunner::validate_version("1.0.0").is_ok());
        assert!(CliRunner::validate_version("2.1.0").is_ok());
        assert!(CliRunner::validate_version("1.0.0-alpha.1").is_ok());
        assert!(CliRunner::validate_version("1.0.0-beta.2").is_ok());
        assert!(CliRunner::validate_version("3.0.0+build.456").is_ok());
        assert!(CliRunner::validate_version("1.0.0-alpha.1+build.123").is_ok());
    }

    #[test]
    fn test_version_validation_invalid() {
        // Invalid semantic versions
        let result = CliRunner::validate_version("1.0");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid version format"));

        let result = CliRunner::validate_version("v1.0.0");
        assert!(result.is_err());

        let result = CliRunner::validate_version("1.0.0.0");
        assert!(result.is_err());

        let result = CliRunner::validate_version("latest");
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_args_registry_url() {
        let args = PublishArgs {
            module_name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            changelog: None,
            artifact_path: PathBuf::from("/path/to/artifact.tar.gz"),
            registry_url: "http://localhost:8091/".to_string(),
            stable: false,
            metadata: None,
        };

        // Should strip trailing slash
        assert_eq!(args.registry_url(), "http://localhost:8091");

        let args2 = PublishArgs {
            module_name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            changelog: None,
            artifact_path: PathBuf::from("/path/to/artifact.tar.gz"),
            registry_url: "http://localhost:8091".to_string(),
            stable: false,
            metadata: None,
        };

        assert_eq!(args2.registry_url(), "http://localhost:8091");
    }

    #[test]
    fn test_publish_args_parse_metadata() {
        let args = PublishArgs {
            module_name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            changelog: None,
            artifact_path: PathBuf::from("/path/to/artifact.tar.gz"),
            registry_url: "http://localhost:8091".to_string(),
            stable: false,
            metadata: Some(r#"{"key": "value"}"#.to_string()),
        };

        let metadata = args.parse_metadata();
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap()["key"], "value");

        let args_no_metadata = PublishArgs {
            module_name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            changelog: None,
            artifact_path: PathBuf::from("/path/to/artifact.tar.gz"),
            registry_url: "http://localhost:8091".to_string(),
            stable: false,
            metadata: None,
        };

        assert!(args_no_metadata.parse_metadata().is_none());
    }

    #[test]
    fn test_publish_args_parse_metadata_invalid_json() {
        let args = PublishArgs {
            module_name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            changelog: None,
            artifact_path: PathBuf::from("/path/to/artifact.tar.gz"),
            registry_url: "http://localhost:8091".to_string(),
            stable: false,
            metadata: Some("not valid json".to_string()),
        };

        // Invalid JSON should return None (not panic)
        assert!(args.parse_metadata().is_none());
    }
}
