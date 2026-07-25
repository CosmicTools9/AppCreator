//! Structure Module Generator Test
//!
//! 基于数据库模型 (zc_id_subj-org, zc_id_leve-post-resp, zc_id_leve-group_member)
//! 生成 Structure (组织架构) 模块的后端和前端代码

use alioth_gen::generator::ir::module::{MetaBusinessRule, MetaEntity, MetaField, MetaFieldType};
use alioth_gen::generator::ir::module::{
    MetaModule, MetaPage, MetaPermission, PageLayout, PageType,
};
use alioth_gen::generator::module::ModuleApiGenerator;
use std::fs;
use std::path::Path;

fn create_structure_module() -> MetaModule {
    let mut module = MetaModule::new("structure");

    // Organization 实体
    let organization_entity = MetaEntity {
        name: "Organization".to_string(),
        description: Some("组织架构实体".to_string()),
        fields: vec![
            MetaField {
                name: "code".to_string(),
                field_type: MetaFieldType::String,
                description: Some("组织编码".to_string()),
                nullable: false,
                unique: true,
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "name".to_string(),
                field_type: MetaFieldType::String,
                description: Some("组织名称".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "org_type".to_string(),
                field_type: MetaFieldType::String,
                description: Some("组织类型".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "parent_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("上级组织ID".to_string()),
                nullable: true,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "level".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("组织层级".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("1".to_string()),
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "status".to_string(),
                field_type: MetaFieldType::String,
                description: Some("状态".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("active".to_string()),
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
        ],
        relations: vec![],
        annotations: vec![],
        parent_classes: vec![],
        equivalent_classes: vec![],
        disjoint_classes: vec![],
        is_abstract: false,
        table_name: None,
        state_machine: Default::default(),
        transitions: vec![],
        lifecycle_hooks: vec![],
        business_rules: vec![MetaBusinessRule {
            name: "positive_level".to_string(),
            condition: "level >= 1".to_string(),
            action: None,
            error_message: Some("组织层级必须大于等于1".to_string()),
            priority: 1,
            trigger: "always".to_string(),
        }],
        swrl_rules: vec![],
        constraints: vec![],
        permission_config: Default::default(),
        permission_inheritance: Default::default(),
        permission_conflict_resolution: Default::default(),
        quality_rules: vec![],
    };

    // Position 实体
    let position_entity = MetaEntity {
        name: "Position".to_string(),
        description: Some("岗位职级实体".to_string()),
        fields: vec![
            MetaField {
                name: "code".to_string(),
                field_type: MetaFieldType::String,
                description: Some("岗位编码".to_string()),
                nullable: false,
                unique: true,
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "name".to_string(),
                field_type: MetaFieldType::String,
                description: Some("岗位名称".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "level".to_string(),
                field_type: MetaFieldType::String,
                description: Some("职级".to_string()),
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "org_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("所属组织ID".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "responsibilities".to_string(),
                field_type: MetaFieldType::String,
                description: Some("岗位职责".to_string()),
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
        ],
        relations: vec![],
        annotations: vec![],
        parent_classes: vec![],
        equivalent_classes: vec![],
        disjoint_classes: vec![],
        is_abstract: false,
        table_name: None,
        state_machine: Default::default(),
        transitions: vec![],
        lifecycle_hooks: vec![],
        business_rules: vec![],
        swrl_rules: vec![],
        constraints: vec![],
        permission_config: Default::default(),
        permission_inheritance: Default::default(),
        permission_conflict_resolution: Default::default(),
        quality_rules: vec![],
    };

    // Group 实体
    let group_entity = MetaEntity {
        name: "Group".to_string(),
        description: Some("群组实体".to_string()),
        fields: vec![
            MetaField {
                name: "code".to_string(),
                field_type: MetaFieldType::String,
                description: Some("群组编码".to_string()),
                nullable: false,
                unique: true,
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "name".to_string(),
                field_type: MetaFieldType::String,
                description: Some("群组名称".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: None,
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "group_type".to_string(),
                field_type: MetaFieldType::String,
                description: Some("群组类型".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("department".to_string()),
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: None,
                max_cardinality: None,
                is_functional: false,
                constraints: vec![],
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
            MetaField {
                name: "description".to_string(),
                field_type: MetaFieldType::String,
                description: Some("群组描述".to_string()),
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
                field_permission: Default::default(),
                throws_clauses: vec![],
                quality_rules: vec![],
            },
        ],
        relations: vec![],
        annotations: vec![],
        parent_classes: vec![],
        equivalent_classes: vec![],
        disjoint_classes: vec![],
        is_abstract: false,
        table_name: None,
        state_machine: Default::default(),
        transitions: vec![],
        lifecycle_hooks: vec![],
        business_rules: vec![],
        swrl_rules: vec![],
        constraints: vec![],
        permission_config: Default::default(),
        permission_inheritance: Default::default(),
        permission_conflict_resolution: Default::default(),
        quality_rules: vec![],
    };

    module.add_entity(organization_entity);
    module.add_entity(position_entity);
    module.add_entity(group_entity);

    // 添加页面
    module.add_page(MetaPage {
        name: "OrganizationList".to_string(),
        page_type: PageType::List,
        entity: "Organization".to_string(),
        layout: PageLayout {
            columns: vec![
                "code".to_string(),
                "name".to_string(),
                "org_type".to_string(),
                "level".to_string(),
                "status".to_string(),
            ],
            filters: vec!["org_type".to_string(), "status".to_string()],
            sections: vec![],
        },
    });

    module.add_page(MetaPage {
        name: "PositionList".to_string(),
        page_type: PageType::List,
        entity: "Position".to_string(),
        layout: PageLayout {
            columns: vec![
                "code".to_string(),
                "name".to_string(),
                "level".to_string(),
                "org_id".to_string(),
            ],
            filters: vec!["level".to_string()],
            sections: vec![],
        },
    });

    module.add_page(MetaPage {
        name: "GroupList".to_string(),
        page_type: PageType::List,
        entity: "Group".to_string(),
        layout: PageLayout {
            columns: vec![
                "code".to_string(),
                "name".to_string(),
                "group_type".to_string(),
                "description".to_string(),
            ],
            filters: vec!["group_type".to_string()],
            sections: vec![],
        },
    });

    // 添加权限
    module.add_permission(MetaPermission {
        role: "hr_manager".to_string(),
        actions: vec![
            "create".to_string(),
            "read".to_string(),
            "update".to_string(),
        ],
    });
    module.add_permission(MetaPermission {
        role: "admin".to_string(),
        actions: vec![
            "create".to_string(),
            "read".to_string(),
            "update".to_string(),
            "delete".to_string(),
        ],
    });

    module
}

fn write_generated_files(
    base_path: &Path,
    output: &alioth_gen::generator::GeneratedOutput,
) -> std::io::Result<()> {
    for file in &output.files {
        let file_path = base_path.join(&file.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &file.content)?;
        println!("  写入: {}", file_path.display());
    }
    Ok(())
}

#[test]
fn test_generate_structure_module() {
    println!("\n========================================");
    println!("Structure Module 生成测试");
    println!("========================================\n");

    let module = create_structure_module();
    println!("✓ 已创建 Structure MetaModule");
    println!("  - 实体数量: {}", module.entities.len());
    println!("  - 页面数量: {}", module.pages.len());
    println!("  - 权限数量: {}", module.permissions.len());

    println!("\n--- 生成后端代码 ---");
    let api_generator = ModuleApiGenerator::new();
    let api_output = api_generator.generate(&module).expect("后端生成失败");
    println!("✓ 后端生成完成 ({} files)", api_output.files.len());

    println!("\n--- 后端生成文件 ---");
    for file in &api_output.files {
        println!("  - {}", file.path.display());
    }

    let backend_files: Vec<_> = api_output
        .files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();
    assert!(backend_files.contains(&"Cargo.toml".to_string()));
    assert!(
        !backend_files.contains(&"src/main.rs".to_string()),
        "Library crate 不应生成 main.rs"
    );
    assert!(backend_files.contains(&"src/lib.rs".to_string()));
    assert!(backend_files.contains(&"src/models/mod.rs".to_string()));
    assert!(backend_files.contains(&"src/models/organization.rs".to_string()));
    assert!(backend_files.contains(&"src/models/position.rs".to_string()));
    assert!(backend_files.contains(&"src/models/group.rs".to_string()));

    println!("\n========================================");
    println!("✅ Structure Module 生成测试通过!");
    println!("========================================\n");
}

#[test]
fn test_write_structure_module_to_disk() {
    println!("\n========================================");
    println!("写入 Structure Module 到磁盘");
    println!("========================================\n");

    let module = create_structure_module();

    let api_generator = ModuleApiGenerator::new();
    let api_output = api_generator.generate(&module).expect("后端生成失败");

    let backend_path = Path::new("../../../Pre-Proc/Alioth/Sources/Modules/structure/backend");
    println!("\n--- 写入后端文件到 {} ---", backend_path.display());
    write_generated_files(backend_path, &api_output).expect("写入后端文件失败");

    println!("\n========================================");
    println!("✅ Structure Module 文件写入完成!");
    println!("========================================\n");
}
