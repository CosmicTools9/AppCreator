//! Inventory Module Generator Test - Phase 50
//!
//! 此测试解析 Inventory DSL 并使用 Module 生成器生成后端和前端代码

use alioth_gen::generator::ir::module::{MetaBusinessRule, MetaEntity, MetaField, MetaFieldType};
use alioth_gen::generator::ir::module::{
    MetaModule, MetaPage, MetaPermission, PageLayout, PageType,
};
use alioth_gen::generator::module::ModuleApiGenerator;
use std::fs;
use std::path::Path;

/// 创建 Inventory 模块的 MetaModule
fn create_inventory_module() -> MetaModule {
    let mut module = MetaModule::new("inventory");

    // 添加 Product 实体
    let product_entity = MetaEntity {
        name: "Product".to_string(),
        description: Some("产品实体".to_string()),
        fields: vec![
            MetaField {
                name: "sku".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品SKU".to_string()),
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
                description: Some("产品名称".to_string()),
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
                name: "description".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品描述".to_string()),
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
            MetaField {
                name: "category".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品类别".to_string()),
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
                name: "unit_price".to_string(),
                field_type: MetaFieldType::Decimal,
                description: Some("单价".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("0".to_string()),
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
            name: "valid_price".to_string(),
            condition: "unit_price >= 0".to_string(),
            action: None,
            error_message: Some("产品价格必须大于等于0".to_string()),
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

    // 添加 Warehouse 实体
    let warehouse_entity = MetaEntity {
        name: "Warehouse".to_string(),
        description: Some("仓库实体".to_string()),
        fields: vec![
            MetaField {
                name: "code".to_string(),
                field_type: MetaFieldType::String,
                description: Some("仓库编码".to_string()),
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
                description: Some("仓库名称".to_string()),
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
                name: "location".to_string(),
                field_type: MetaFieldType::String,
                description: Some("仓库位置".to_string()),
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
                name: "manager".to_string(),
                field_type: MetaFieldType::String,
                description: Some("仓库管理员".to_string()),
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

    // 添加 Inventory 实体
    let inventory_entity = MetaEntity {
        name: "Inventory".to_string(),
        description: Some("库存实体".to_string()),
        fields: vec![
            MetaField {
                name: "product_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品ID".to_string()),
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
                name: "warehouse_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("仓库ID".to_string()),
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
                name: "quantity".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("库存数量".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("0".to_string()),
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
                name: "reserved".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("预留数量".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("0".to_string()),
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
                name: "available".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("可用数量".to_string()),
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
        business_rules: vec![
            MetaBusinessRule {
                name: "available_computed".to_string(),
                condition: "quantity - reserved".to_string(),
                action: None,
                error_message: Some("可用数量 = 库存数量 - 预留数量".to_string()),
                priority: 1,
                trigger: "always".to_string(),
            },
            MetaBusinessRule {
                name: "non_negative".to_string(),
                condition: "quantity >= 0 AND reserved >= 0".to_string(),
                action: None,
                error_message: Some("库存数量和预留数量不能为负数".to_string()),
                priority: 1,
                trigger: "always".to_string(),
            },
            MetaBusinessRule {
                name: "valid_reserved".to_string(),
                condition: "reserved <= quantity".to_string(),
                action: None,
                error_message: Some("预留数量不能大于库存数量".to_string()),
                priority: 1,
                trigger: "always".to_string(),
            },
        ],
        swrl_rules: vec![],
        constraints: vec![],
        permission_config: Default::default(),
        permission_inheritance: Default::default(),
        permission_conflict_resolution: Default::default(),
        quality_rules: vec![],
    };

    // 添加 StockMovement 实体
    let stock_movement_entity = MetaEntity {
        name: "StockMovement".to_string(),
        description: Some("库存移动记录实体".to_string()),
        fields: vec![
            MetaField {
                name: "inventory_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("库存ID".to_string()),
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
                name: "movement_type".to_string(),
                field_type: MetaFieldType::String,
                description: Some("移动类型".to_string()),
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
                name: "quantity".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("移动数量".to_string()),
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
                name: "reason".to_string(),
                field_type: MetaFieldType::String,
                description: Some("移动原因".to_string()),
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
            MetaField {
                name: "created_at".to_string(),
                field_type: MetaFieldType::DateTime,
                description: Some("创建时间".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("NOW()".to_string()),
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
            name: "positive_quantity".to_string(),
            condition: "quantity != 0".to_string(),
            action: None,
            error_message: Some("移动数量不能为0".to_string()),
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

    module.add_entity(product_entity);
    module.add_entity(warehouse_entity);
    module.add_entity(inventory_entity);
    module.add_entity(stock_movement_entity);

    // 添加页面
    module.add_page(MetaPage {
        name: "ProductList".to_string(),
        page_type: PageType::List,
        entity: "Product".to_string(),
        layout: PageLayout {
            columns: vec![
                "sku".to_string(),
                "name".to_string(),
                "category".to_string(),
                "unit_price".to_string(),
            ],
            filters: vec!["category".to_string()],
            sections: vec![],
        },
    });

    module.add_page(MetaPage {
        name: "InventoryDashboard".to_string(),
        page_type: PageType::Dashboard,
        entity: "Inventory".to_string(),
        layout: PageLayout {
            columns: vec![],
            filters: vec![],
            sections: vec![
                "low_stock_alert".to_string(),
                "total_value".to_string(),
                "movement_trend".to_string(),
            ],
        },
    });

    module.add_page(MetaPage {
        name: "InventoryList".to_string(),
        page_type: PageType::List,
        entity: "Inventory".to_string(),
        layout: PageLayout {
            columns: vec![
                "product_id".to_string(),
                "warehouse_id".to_string(),
                "quantity".to_string(),
                "reserved".to_string(),
                "available".to_string(),
            ],
            filters: vec!["warehouse_id".to_string()],
            sections: vec![],
        },
    });

    module.add_page(MetaPage {
        name: "WarehouseList".to_string(),
        page_type: PageType::List,
        entity: "Warehouse".to_string(),
        layout: PageLayout {
            columns: vec![
                "code".to_string(),
                "name".to_string(),
                "location".to_string(),
                "manager".to_string(),
            ],
            filters: vec![],
            sections: vec![],
        },
    });

    // 添加权限
    module.add_permission(MetaPermission {
        role: "inventory_manager".to_string(),
        actions: vec![
            "create".to_string(),
            "read".to_string(),
            "update".to_string(),
        ],
    });
    module.add_permission(MetaPermission {
        role: "sales".to_string(),
        actions: vec!["read".to_string()],
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

/// 将生成的文件写入磁盘
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
fn test_generate_inventory_module() {
    println!("\n========================================");
    println!("Phase 50: Inventory Module 生成测试");
    println!("========================================\n");

    // 创建 inventory 模块
    let module = create_inventory_module();
    println!("✓ 已创建 Inventory MetaModule");
    println!("  - 实体数量: {}", module.entities.len());
    println!("  - 页面数量: {}", module.pages.len());
    println!("  - 权限数量: {}", module.permissions.len());

    // 生成后端代码
    println!("\n--- 生成后端代码 ---");
    let api_generator = ModuleApiGenerator::new();
    let api_output = api_generator.generate(&module).expect("后端生成失败");
    println!("✓ 后端生成完成 ({} files)", api_output.files.len());

    // 输出文件列表
    println!("\n--- 后端生成文件 ---");
    for file in &api_output.files {
        println!("  - {}", file.path.display());
    }

    // 验证生成的文件
    println!("\n--- 验证生成结果 ---");

    // 验证后端关键文件
    let backend_files: Vec<_> = api_output
        .files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();
    assert!(
        backend_files.contains(&"Cargo.toml".to_string()),
        "缺少 Cargo.toml"
    );
    assert!(
        !backend_files.contains(&"src/main.rs".to_string()),
        "Library crate 不应生成 main.rs"
    );
    assert!(
        backend_files.contains(&"src/lib.rs".to_string()),
        "缺少 lib.rs"
    );
    assert!(
        backend_files.contains(&"src/routes.rs".to_string()),
        "缺少 routes.rs"
    );
    assert!(
        backend_files.contains(&"src/errors.rs".to_string()),
        "缺少 errors.rs"
    );
    assert!(
        backend_files.contains(&"src/models/mod.rs".to_string()),
        "缺少 models/mod.rs"
    );
    assert!(
        backend_files.contains(&"src/models/product.rs".to_string()),
        "缺少 models/product.rs"
    );
    assert!(
        backend_files.contains(&"src/models/warehouse.rs".to_string()),
        "缺少 models/warehouse.rs"
    );
    assert!(
        backend_files.contains(&"src/models/inventory.rs".to_string()),
        "缺少 models/inventory.rs"
    );
    assert!(
        backend_files.contains(&"src/models/stock_movement.rs".to_string()),
        "缺少 models/stock_movement.rs"
    );

    println!("✓ 后端关键文件验证通过");

    println!("\n========================================");
    println!("✅ Inventory Module 生成测试通过!");
    println!("========================================\n");
}

#[test]
fn test_write_inventory_module_to_disk() {
    println!("\n========================================");
    println!("Phase 50: 写入 Inventory Module 到磁盘");
    println!("========================================\n");

    // 创建 inventory 模块
    let module = create_inventory_module();

    // 生成后端代码
    let api_generator = ModuleApiGenerator::new();
    let api_output = api_generator.generate(&module).expect("后端生成失败");

    // 写入后端文件
    let backend_path = Path::new("../../../Pre-Proc/Alioth/Sources/Modules/inventory/backend");
    println!("\n--- 写入后端文件到 {} ---", backend_path.display());
    write_generated_files(backend_path, &api_output).expect("写入后端文件失败");

    println!("\n========================================");
    println!("✅ Inventory Module 文件写入完成!");
    println!("========================================\n");
}

#[test]
fn test_inventory_module_compilation() {
    // 此测试验证生成器可以正确处理模块定义
    let module = create_inventory_module();

    let api_generator = ModuleApiGenerator::new();

    // 验证 API 生成器能正常处理模块
    let api_result = api_generator.generate(&module);

    assert!(api_result.is_ok(), "API 生成器应成功处理 Inventory 模块");

    // 验证生成的文件数量
    let api_output = api_result.unwrap();

    assert!(api_output.files.len() >= 11, "后端应生成至少 11 个文件");
    assert!(
        !api_output
            .files
            .iter()
            .any(|f| f.path == std::path::PathBuf::from("src/auth/middleware.rs")),
        "禁止生成 src/auth/middleware.rs：auth 中间件须由 Gateway 统一处理"
    );
}
