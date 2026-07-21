//! Orders Module File Writer - Phase 49
//!
//! 此程序将生成的 Orders 模块文件写入 Modules/orders/ 目录

use alioth_gen::generator::ir::module::{
    MetaBusinessRule, MetaEntity, MetaField, MetaFieldType, MetaStateMachine,
};
use alioth_gen::generator::ir::module::{
    MetaModule, MetaPage, MetaPermission, PageLayout, PageType,
};
use alioth_gen::generator::module::ModuleApiGenerator;
use std::fs;
use std::path::Path;

/// Orders 模块输出目录
const BACKEND_OUTPUT_DIR: &str = "/Users/alioth/Git-workspace/AliothStudio/Modules/orders/backend";
/// 创建 Orders 模块的 MetaModule
fn create_orders_module() -> MetaModule {
    let mut module = MetaModule::new("orders");

    // Order 实体
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

    // OrderItem 实体
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

    // 页面
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

    // 权限
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
fn write_generated_files(
    base_path: &str,
    output: &alioth_gen::generator::GeneratedOutput,
) -> std::io::Result<()> {
    let base = Path::new(base_path);

    for file in &output.files {
        let file_path = base.join(&file.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &file.content)?;
        println!("  写入: {}", file_path.display());
    }
    Ok(())
}

fn main() {
    println!("\n========================================");
    println!("Phase 49: Orders Module 文件生成");
    println!("========================================\n");

    // 创建 orders 模块
    let module = create_orders_module();
    println!("✓ 已创建 Orders MetaModule");
    println!("  - 实体数量: {}", module.entities.len());
    println!("  - 页面数量: {}", module.pages.len());
    println!("  - 权限数量: {}", module.permissions.len());

    // 清理旧的生成文件（保留目录结构）
    println!("\n--- 清理旧文件 ---");
    let backend_path = Path::new(BACKEND_OUTPUT_DIR);

    if backend_path.exists() {
        for entry in fs::read_dir(backend_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path).unwrap();
            } else if path.is_dir() && path.file_name().map(|n| n != "target") == Some(true) {
                fs::remove_dir_all(path).unwrap();
            }
        }
        println!("  已清理 backend 目录");
    }

    // 生成后端代码
    println!("\n--- 生成后端代码 ---");
    let api_generator = ModuleApiGenerator::new();
    let api_output = api_generator.generate(&module).expect("后端生成失败");
    println!("✓ 后端生成完成 ({} files)", api_output.files.len());

    // 写入后端文件
    println!("\n--- 写入后端文件 ---");
    write_generated_files(BACKEND_OUTPUT_DIR, &api_output).expect("写入后端文件失败");

    println!("\n========================================");
    println!("✅ Orders Module 文件生成完成!");
    println!("========================================\n");
    println!("生成的文件:");
    println!("  Backend: {}/", BACKEND_OUTPUT_DIR);
}
