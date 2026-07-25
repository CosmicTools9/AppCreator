use alioth_gen::generator::ir::module::{
    MetaEntity, MetaField, MetaFieldType, MetaModule, MetaPage, MetaPermission, PageLayout,
    PageType,
};
use alioth_gen::generator::module::ModuleApiGenerator;

#[test]
fn test_ir_to_module_pipeline() {
    let mut module = MetaModule::new("inventory");

    let product_entity = MetaEntity {
        name: "Product".to_string(),
        description: Some("产品实体".to_string()),
        fields: vec![
            MetaField {
                name: "code".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品编码".to_string()),
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
                name: "notice".to_string(),
                field_type: MetaFieldType::String,
                description: Some("产品名称".to_string()),
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
        business_rules: vec![],
        swrl_rules: vec![],
        constraints: vec![],
        permission_config: Default::default(),
        permission_inheritance: Default::default(),
        permission_conflict_resolution: Default::default(),
        quality_rules: vec![],
    };

    module.add_entity(product_entity);
    module.add_page(MetaPage {
        name: "ProductList".to_string(),
        page_type: PageType::List,
        entity: "Product".to_string(),
        layout: PageLayout {
            columns: vec![
                "code".to_string(),
                "notice".to_string(),
                "unit_price".to_string(),
            ],
            filters: vec![],
            sections: vec![],
        },
    });
    module.add_permission(MetaPermission {
        role: "admin".to_string(),
        actions: vec![
            "create".to_string(),
            "read".to_string(),
            "update".to_string(),
        ],
    });

    // MetaModule → Module 代码
    let api_gen = ModuleApiGenerator::new();
    let api_output = api_gen.generate(&module).unwrap();

    // 验证输出包含关键文件
    assert!(api_output
        .files
        .iter()
        .any(|f| f.path.to_string_lossy() == "Cargo.toml"));
    assert!(api_output
        .files
        .iter()
        .any(|f| f.path.to_string_lossy() == "src/lib.rs"));
}
