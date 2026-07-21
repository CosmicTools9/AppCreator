//! 命名与类型映射公共函数
//!
//! 统一 GeneratorModel 构建过程中的命名转换和类型映射，
//! 确保 composer、ontology_transformer、generate.rs 等路径的一致性。

use super::{EntityName, FieldName, GeneratorFieldType};

/// 从原始名称构建 EntityName（含各种命名变体）
pub fn build_entity_name(raw: &str) -> EntityName {
    use convert_case::{Case, Casing};
    let snake = raw.to_case(Case::Snake);
    let pascal = raw.to_case(Case::Pascal);
    let kebab = raw.to_case(Case::Kebab);
    let plural = pluralizer::pluralize(raw, 2, false);
    EntityName {
        raw: raw.to_string(),
        snake: snake.clone(),
        camel: raw.to_case(Case::Camel),
        pascal: pascal.clone(),
        kebab: kebab.clone(),
        screaming_snake: raw.to_case(Case::UpperSnake),
        plural_snake: plural.to_case(Case::Snake),
        plural_pascal: plural.to_case(Case::Pascal),
        plural_kebab: plural.to_case(Case::Kebab),
    }
}

/// 从原始名称构建 FieldName
pub fn build_field_name(raw: &str) -> FieldName {
    use convert_case::{Case, Casing};
    FieldName {
        raw: raw.to_string(),
        snake: raw.to_case(Case::Snake),
        camel: raw.to_case(Case::Camel),
        pascal: raw.to_case(Case::Pascal),
    }
}

/// 将数据库/本体类型字符串映射为 GeneratorFieldType
pub fn map_field_type(dt: &str) -> GeneratorFieldType {
    let lower = dt.to_lowercase();
    match lower.as_str() {
        "text" | "string" | "varchar" | "char" => GeneratorFieldType::Text,
        "integer" | "int" | "int4" | "i32" => GeneratorFieldType::Integer,
        "bigint" | "int8" | "long" | "i64" => GeneratorFieldType::BigInt,
        "decimal" | "numeric" | "float" | "number" => GeneratorFieldType::Decimal,
        "double" | "real" => GeneratorFieldType::Decimal,
        "boolean" | "bool" => GeneratorFieldType::Boolean,
        "timestamp" | "datetime" | "date" | "time" => GeneratorFieldType::DateTime,
        "uuid" => GeneratorFieldType::Uuid,
        "json" | "jsonb" | "array" | "object" => GeneratorFieldType::Json,
        other => {
            if let Some(stripped) = dt.strip_prefix("enum:") {
                GeneratorFieldType::Enum(stripped.to_string())
            } else if let Some(stripped) = dt.strip_prefix("ref:") {
                GeneratorFieldType::Reference(stripped.to_string())
            } else if let Some(stripped) = dt.strip_prefix("reference:") {
                GeneratorFieldType::Reference(stripped.to_string())
            } else {
                common::telemetry::warn!("Unknown field type '{}', defaulting to Text", other);
                GeneratorFieldType::Text
            }
        }
    }
}
