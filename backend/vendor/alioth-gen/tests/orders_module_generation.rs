//! Orders Module Generator Test - Phase 49
//!
//! 此测试解析 Orders DSL 并使用 Module 生成器生成后端和前端代码

use alioth_gen::generator::ir::module::{
    MetaBusinessRule, MetaEntity, MetaField, MetaFieldType, MetaStateMachine,
};
use alioth_gen::generator::ir::module::{
    MetaModule, MetaPage, MetaPermission, PageLayout, PageType,
};
use alioth_gen::generator::module::ModuleApiGenerator;

/// 创建 Orders 模块的 MetaModule
fn create_orders_module() -> MetaModule {
    let mut module = MetaModule::new("orders");

    // 添加 Order 实体
    let order_entity = MetaEntity {
        name: "Order".to_string(),
        description: Some("订单实体".to_string()),
        fields: vec![
            MetaField {
                name: "order_number".to_string(),
                field_type: MetaFieldType::String,
                description: Some("订单编号".to_string()),
                nullable: false,
                unique: true,
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
                name: "customer_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("客户ID".to_string()),
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
                name: "status".to_string(),
                field_type: MetaFieldType::String,
                description: Some("订单状态".to_string()),
                nullable: false,
                unique: false,
                indexed: true,
                default_value: Some("Draft".to_string()),
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
                name: "total_amount".to_string(),
                field_type: MetaFieldType::Decimal,
                description: Some("总金额".to_string()),
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
                name: "shipping_address".to_string(),
                field_type: MetaFieldType::String,
                description: Some("收货地址".to_string()),
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
        state_machine: MetaStateMachine {
            enabled: true,
            states: vec![
                "Draft".to_string(),
                "Submitted".to_string(),
                "Paid".to_string(),
                "Shipped".to_string(),
                "Delivered".to_string(),
                "Cancelled".to_string(),
            ],
            initial_state: Some("Draft".to_string()),
            state_field: Some("status".to_string()),
        },
        transitions: vec![],
        lifecycle_hooks: vec![],
        business_rules: vec![MetaBusinessRule {
            name: "total_must_match".to_string(),
            condition: "sum(items.subtotal) == total_amount".to_string(),
            action: None,
            error_message: Some("订单金额必须与订单项小计之和匹配".to_string()),
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

    // 添加 OrderItem 实体
    let order_item_entity = MetaEntity {
        name: "OrderItem".to_string(),
        description: Some("订单项实体".to_string()),
        fields: vec![
            MetaField {
                name: "order_id".to_string(),
                field_type: MetaFieldType::String,
                description: Some("订单ID".to_string()),
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
                name: "quantity".to_string(),
                field_type: MetaFieldType::Integer,
                description: Some("数量".to_string()),
                nullable: false,
                unique: false,
                indexed: false,
                default_value: Some("1".to_string()),
                validations: vec![],
                annotations: vec![],
                domain: None,
                range: None,
                min_cardinality: Some(1),
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
                name: "subtotal".to_string(),
                field_type: MetaFieldType::Decimal,
                description: Some("小计".to_string()),
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
            name: "subtotal_computed".to_string(),
            condition: "quantity * unit_price".to_string(),
            action: None,
            error_message: Some("小计必须等于数量乘以单价".to_string()),
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

    module.add_entity(order_entity);
    module.add_entity(order_item_entity);

    // 添加 OrderList 页面
    module.add_page(MetaPage {
        name: "OrderList".to_string(),
        page_type: PageType::List,
        entity: "Order".to_string(),
        layout: PageLayout {
            columns: vec![
                "order_number".to_string(),
                "customer_id".to_string(),
                "status".to_string(),
                "total_amount".to_string(),
            ],
            filters: vec!["status".to_string()],
            sections: vec![],
        },
    });

    // 添加 OrderDetail 页面
    module.add_page(MetaPage {
        name: "OrderDetail".to_string(),
        page_type: PageType::Detail,
        entity: "Order".to_string(),
        layout: PageLayout {
            columns: vec![],
            filters: vec![],
            sections: vec![
                "header".to_string(),
                "items".to_string(),
                "payment".to_string(),
                "shipping".to_string(),
            ],
        },
    });

    // 添加权限
    module.add_permission(MetaPermission {
        role: "sales".to_string(),
        actions: vec![
            "create".to_string(),
            "read".to_string(),
            "update".to_string(),
        ],
    });
    module.add_permission(MetaPermission {
        role: "finance".to_string(),
        actions: vec!["read".to_string(), "update_status".to_string()],
    });
    module.add_permission(MetaPermission {
        role: "admin".to_string(),
        actions: vec!["delete".to_string()],
    });

    module
}

/// 将生成的文件写入磁盘
#[test]
fn test_generate_orders_module() {
    println!("\n========================================");
    println!("Phase 49: Orders Module 生成测试");
    println!("========================================\n");

    // 创建 orders 模块
    let module = create_orders_module();
    println!("✓ 已创建 Orders MetaModule");
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
        backend_files.contains(&"src/models/order.rs".to_string()),
        "缺少 models/order.rs"
    );
    assert!(
        backend_files.contains(&"src/models/order_item.rs".to_string()),
        "缺少 models/order_item.rs"
    );

    println!("✓ 后端关键文件验证通过");

    println!("\n========================================");
    println!("✅ Orders Module 生成测试通过!");
    println!("========================================\n");
}

#[test]
fn test_orders_module_compilation() {
    // 此测试验证生成器可以正确处理模块定义
    let module = create_orders_module();

    let api_generator = ModuleApiGenerator::new();

    // 验证 API 生成器能正常处理模块
    let api_result = api_generator.generate(&module);

    assert!(api_result.is_ok(), "API 生成器应成功处理 Orders 模块");

    // 验证生成的文件数量
    let api_output = api_result.unwrap();

    assert!(api_output.files.len() >= 9, "后端应生成至少 9 个文件");
    assert!(
        !api_output
            .files
            .iter()
            .any(|f| f.path == std::path::PathBuf::from("src/auth/middleware.rs")),
        "禁止生成 src/auth/middleware.rs：auth 中间件须由 Gateway 统一处理"
    );
}
