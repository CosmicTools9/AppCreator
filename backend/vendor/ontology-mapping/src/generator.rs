use crate::output::{FieldMapping, MappedEntity, MappingOutput};

pub struct CodeGenerator;

impl CodeGenerator {
    /// Generate a DTO struct for the given mapped entity.
    pub fn generate_dto(entity: &MappedEntity) -> String {
        let mut out = String::new();
        out.push_str("// === DTO: ");
        out.push_str(&entity.name);
        out.push_str(" ===\n");
        out.push_str("// Generated from ontology-mapper v0.1.0\n");
        out.push('\n');
        out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        out.push_str("pub struct ");
        out.push_str(&entity.name);
        out.push_str("Dto {\n");

        for field in &entity.fields {
            let json_field = Self::safe_field_name(&field.json_path);
            let rust_type = Self::column_to_rust_type(field);
            out.push_str("    pub ");
            out.push_str(&json_field);
            out.push_str(": Option<");
            out.push_str(&rust_type);
            out.push_str(">,");
            if let Some(scalar) = &field.scalar_table {
                out.push_str("  // scalar: ");
                if let Some(col) = &field.column {
                    out.push_str(col);
                    out.push('-');
                }
                out.push_str(scalar);
            }
            out.push('\n');
        }

        out.push_str("}\n");
        out
    }

    /// Generate a service struct with CRUD method stubs.
    pub fn generate_service(entity: &MappedEntity) -> String {
        let mut out = String::new();
        out.push_str("// === Service: ");
        out.push_str(&entity.name);
        out.push_str(" ===\n");
        out.push_str("// Generated from ontology-mapper v0.1.0\n");
        out.push('\n');
        out.push_str("use sqlx::PgPool;\n");
        out.push_str("use anyhow::Result;\n");
        out.push('\n');
        out.push_str("pub struct ");
        out.push_str(&entity.name);
        out.push_str("Service {\n");
        out.push_str("    pool: PgPool,\n");
        out.push_str("}\n");
        out.push('\n');
        out.push_str("impl ");
        out.push_str(&entity.name);
        out.push_str("Service {\n");
        out.push_str("    pub fn new(pool: PgPool) -> Self {\n");
        out.push_str("        Self { pool }\n");
        out.push_str("    }\n");
        out.push('\n');
        out.push_str("    pub async fn list(&self) -> Result<Vec<");
        out.push_str(&entity.name);
        out.push_str("Dto>> {\n");
        out.push_str("        todo!()\n");
        out.push_str("    }\n");
        out.push('\n');
        out.push_str("    pub async fn get(&self, id: i64) -> Result<Option<");
        out.push_str(&entity.name);
        out.push_str("Dto>> {\n");
        out.push_str("        todo!()\n");
        out.push_str("    }\n");
        out.push('\n');
        out.push_str("    pub async fn create(&self, input: ");
        out.push_str(&entity.name);
        out.push_str("Dto) -> Result<");
        out.push_str(&entity.name);
        out.push_str("Dto> {\n");
        out.push_str("        todo!()\n");
        out.push_str("    }\n");
        out.push('\n');
        out.push_str("    pub async fn update(&self, id: i64, input: ");
        out.push_str(&entity.name);
        out.push_str("Dto) -> Result<");
        out.push_str(&entity.name);
        out.push_str("Dto> {\n");
        out.push_str("        todo!()\n");
        out.push_str("    }\n");
        out.push('\n');
        out.push_str("    pub async fn delete(&self, id: i64) -> Result<()> {\n");
        out.push_str("        todo!()\n");
        out.push_str("    }\n");
        out.push_str("}\n");
        out
    }

    /// Generate Actix-web handler function stubs.
    pub fn generate_handler(entity: &MappedEntity) -> String {
        let mut out = String::new();
        out.push_str("// === Handler: ");
        out.push_str(&entity.name);
        out.push_str(" ===\n");
        out.push_str("// Generated from ontology-mapper v0.1.0\n");
        out.push('\n');
        out.push_str("use actix_web::{web, HttpResponse, Responder};\n");
        out.push_str("use sqlx::PgPool;\n");
        out.push_str("use anyhow::Result;\n");
        out.push('\n');
        out.push_str("pub async fn list_");
        out.push_str(&Self::to_snake_case(&entity.name));
        out.push_str("(pool: web::Data<PgPool>) -> Result<HttpResponse> {\n");
        out.push_str("    let svc = ");
        out.push_str(&entity.name);
        out.push_str("Service::new(pool.get_ref().clone());\n");
        out.push_str("    let items = svc.list().await?;\n");
        out.push_str("    Ok(HttpResponse::Ok().json(items))\n");
        out.push_str("}\n");
        out.push('\n');
        out.push_str("pub async fn get_");
        out.push_str(&Self::to_snake_case(&entity.name));
        out.push_str("(pool: web::Data<PgPool>, path: web::Path<i64>) -> Result<HttpResponse> {\n");
        out.push_str("    let svc = ");
        out.push_str(&entity.name);
        out.push_str("Service::new(pool.get_ref().clone());\n");
        out.push_str("    let id = path.into_inner();\n");
        out.push_str("    let item = svc.get(id).await?;\n");
        out.push_str("    match item {\n");
        out.push_str("        Some(dto) => Ok(HttpResponse::Ok().json(dto)),\n");
        out.push_str("        None => Ok(HttpResponse::NotFound().finish()),\n");
        out.push_str("    }\n");
        out.push_str("}\n");
        out.push('\n');
        out.push_str("pub async fn create_");
        out.push_str(&Self::to_snake_case(&entity.name));
        out.push_str("(pool: web::Data<PgPool>, body: web::Json<");
        out.push_str(&entity.name);
        out.push_str("Dto>) -> Result<HttpResponse> {\n");
        out.push_str("    let svc = ");
        out.push_str(&entity.name);
        out.push_str("Service::new(pool.get_ref().clone());\n");
        out.push_str("    let input = body.into_inner();\n");
        out.push_str("    let created = svc.create(input).await?;\n");
        out.push_str("    Ok(HttpResponse::Created().json(created))\n");
        out.push_str("}\n");
        out.push('\n');
        out.push_str("pub async fn update_");
        out.push_str(&Self::to_snake_case(&entity.name));
        out.push_str("(pool: web::Data<PgPool>, path: web::Path<i64>, body: web::Json<");
        out.push_str(&entity.name);
        out.push_str("Dto>) -> Result<HttpResponse> {\n");
        out.push_str("    let svc = ");
        out.push_str(&entity.name);
        out.push_str("Service::new(pool.get_ref().clone());\n");
        out.push_str("    let id = path.into_inner();\n");
        out.push_str("    let input = body.into_inner();\n");
        out.push_str("    let updated = svc.update(id, input).await?;\n");
        out.push_str("    Ok(HttpResponse::Ok().json(updated))\n");
        out.push_str("}\n");
        out.push('\n');
        out.push_str("pub async fn delete_");
        out.push_str(&Self::to_snake_case(&entity.name));
        out.push_str("(pool: web::Data<PgPool>, path: web::Path<i64>) -> Result<HttpResponse> {\n");
        out.push_str("    let svc = ");
        out.push_str(&entity.name);
        out.push_str("Service::new(pool.get_ref().clone());\n");
        out.push_str("    let id = path.into_inner();\n");
        out.push_str("    svc.delete(id).await?;\n");
        out.push_str("    Ok(HttpResponse::NoContent().finish())\n");
        out.push_str("}\n");
        out
    }

    /// Generate all DTO, service, and handler code for the full mapping output.
    pub fn generate_all(output: &MappingOutput) -> String {
        let mut result = String::new();
        for entity in &output.entities {
            result.push('\n');
            result.push_str(&Self::generate_dto(entity));
            result.push('\n');
            result.push_str(&Self::generate_service(entity));
            result.push('\n');
            result.push_str(&Self::generate_handler(entity));
        }
        result
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Map a column name to its Rust type.
    fn column_to_rust_type(field: &FieldMapping) -> String {
        // If the column is None, use serde_json::Value
        let col = match &field.column {
            Some(c) => c.as_str(),
            None => return "serde_json::Value".to_string(),
        };

        // Scalar references → i64
        if field.scalar_table.is_some() {
            return "i64".to_string();
        }

        match col {
            "id" => return "i64".to_string(),
            "enable" | "flag" => return "bool".to_string(),
            "created_at" | "updated_at" | "deleted_at" | "qk_date" => {
                return "chrono::DateTime<Utc>".to_string()
            }
            _ => {}
        }

        // Prefix-based
        if col.starts_with("fk_") || col.starts_with("qk_") {
            return "i64".to_string();
        }
        if col.starts_with("sk_") || col.starts_with("tk_") || col.starts_with("ck_") {
            return "String".to_string();
        }

        // Name-based
        match col {
            "o_number" | "notice" | "code" | "number" | "comments" | "attachments" | "_t_"
            | "_f_" | "subject" => "String".to_string(),
            _ => "serde_json::Value".to_string(),
        }
    }

    /// Convert a JSON field name to a valid Rust field identifier.
    fn safe_field_name(name: &str) -> String {
        if name == "type" {
            return "r#type".to_string();
        }
        name.replace(['-', '.', ' '], "_")
    }

    /// Convert PascalCase to snake_case for use in function names.
    fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        for (i, ch) in name.char_indices() {
            if ch.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                for lower in ch.to_lowercase() {
                    result.push(lower);
                }
            } else {
                result.push(ch);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::*;

    fn make_field(path: &str, column: Option<&str>, scalar: Option<&str>) -> FieldMapping {
        FieldMapping {
            json_path: path.to_string(),
            column: column.map(String::from),
            scalar_table: scalar.map(String::from),
            ref_table: None,
            tier: Tier::Safe,
            confidence: 1.0,
            source: "test".to_string(),
            alternatives: vec![],
        }
    }

    fn make_entity(name: &str, fields: Vec<FieldMapping>) -> MappedEntity {
        MappedEntity {
            name: name.to_string(),
            mapping: EntityMapping {
                table: String::new(),
                inherits: None,
                source: "test".to_string(),
                tier: Tier::Safe,
                confidence: 1.0,
            },
            coordinates: Coordinates {
                scene: TieredValue {
                    value: "FE".into(),
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "test".into(),
                },
                factor: TieredValue {
                    value: "FA".into(),
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "test".into(),
                },
                function: TieredValue {
                    value: "↓_GD".into(),
                    tier: Tier::Safe,
                    confidence: 1.0,
                    source: "test".into(),
                },
            },
            fields,
            relationships: vec![],
        }
    }

    #[test]
    fn test_generate_dto_basic() {
        let fields = vec![
            make_field("id", Some("id"), None),
            make_field("name", Some("notice"), None),
            make_field("code", Some("code"), None),
        ];
        let entity = make_entity("Agreement", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub struct AgreementDto {"));
        assert!(dto.contains("pub id: Option<i64>"));
        assert!(dto.contains("pub name: Option<String>"));
        assert!(dto.contains("pub code: Option<String>"));
        assert!(dto.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"));
    }

    #[test]
    fn test_generate_dto_scalar_field() {
        let fields = vec![make_field(
            "amount",
            Some("qk_amount"),
            Some("zc_id_scal-amount"),
        )];
        let entity = make_entity("Agreement", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub amount: Option<i64>"));
        assert!(dto.contains("// scalar: qk_amount-zc_id_scal-amount"));
    }

    #[test]
    fn test_generate_dto_foreign_key() {
        let fields = vec![
            make_field("status_id", Some("fk_status"), None),
            make_field("category_id", Some("qk_category"), None),
        ];
        let entity = make_entity("Agreement", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub status_id: Option<i64>"));
        assert!(dto.contains("pub category_id: Option<i64>"));
    }

    #[test]
    fn test_generate_dto_string_prefixes() {
        let fields = vec![
            make_field("currency", Some("sk_currency"), None),
            make_field("tags", Some("tk_tags"), None),
            make_field("category", Some("ck_category"), None),
        ];
        let entity = make_entity("Product", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub currency: Option<String>"));
        assert!(dto.contains("pub tags: Option<String>"));
        assert!(dto.contains("pub category: Option<String>"));
    }

    #[test]
    fn test_generate_dto_boolean() {
        let fields = vec![
            make_field("enable", Some("enable"), None),
            make_field("flag", Some("flag"), None),
        ];
        let entity = make_entity("Setting", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub enable: Option<bool>"));
        assert!(dto.contains("pub flag: Option<bool>"));
    }

    #[test]
    fn test_generate_dto_datetime() {
        let fields = vec![
            make_field("created_at", Some("created_at"), None),
            make_field("updated_at", Some("updated_at"), None),
        ];
        let entity = make_entity("AuditLog", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub created_at: Option<chrono::DateTime<Utc>>"));
        assert!(dto.contains("pub updated_at: Option<chrono::DateTime<Utc>>"));
    }

    #[test]
    fn test_generate_dto_no_column() {
        let fields = vec![make_field("unknown", None, None)];
        let entity = make_entity("Generic", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub unknown: Option<serde_json::Value>"));
    }

    #[test]
    fn test_generate_dto_type_keyword() {
        let fields = vec![make_field("type", Some("_t_"), None)];
        let entity = make_entity("Item", fields);
        let dto = CodeGenerator::generate_dto(&entity);

        assert!(dto.contains("pub r#type: Option<String>"));
    }

    #[test]
    fn test_generate_service_basic() {
        let entity = make_entity("Order", vec![make_field("id", Some("id"), None)]);
        let svc = CodeGenerator::generate_service(&entity);

        assert!(svc.contains("pub struct OrderService"));
        assert!(svc.contains("pub fn new(pool: PgPool)"));
        assert!(svc.contains("pub async fn list(&self) -> Result<Vec<OrderDto>>"));
        assert!(svc.contains("pub async fn get(&self, id: i64) -> Result<Option<OrderDto>>"));
        assert!(svc.contains("pub async fn create(&self, input: OrderDto) -> Result<OrderDto>"));
        assert!(svc
            .contains("pub async fn update(&self, id: i64, input: OrderDto) -> Result<OrderDto>"));
        assert!(svc.contains("pub async fn delete(&self, id: i64) -> Result<()>"));
        assert!(svc.contains("use sqlx::PgPool;"));
        assert!(svc.contains("use anyhow::Result;"));
    }

    #[test]
    fn test_generate_handler_basic() {
        let entity = make_entity("Invoice", vec![make_field("id", Some("id"), None)]);
        let hdl = CodeGenerator::generate_handler(&entity);

        assert!(hdl.contains("pub async fn list_invoice"));
        assert!(hdl.contains("pub async fn get_invoice"));
        assert!(hdl.contains("pub async fn create_invoice"));
        assert!(hdl.contains("pub async fn update_invoice"));
        assert!(hdl.contains("pub async fn delete_invoice"));
        assert!(hdl.contains("InvoiceService"));
        assert!(hdl.contains("web::Data<PgPool>"));
    }

    #[test]
    fn test_generate_all_includes_all_sections() {
        let fields = vec![make_field("name", Some("notice"), None)];
        let entity1 = make_entity("Agreement", fields);
        let entity2 = make_entity("Order", vec![make_field("id", Some("id"), None)]);

        let output = MappingOutput {
            meta: OutputMeta {
                tool_version: "0.1.0".into(),
                alioth_model: "10.0.0".into(),
            },
            entities: vec![entity1, entity2],
            summary: TierSummary {
                safe: 0,
                suggest: 0,
                unclear: 0,
            },
        };

        let generated = CodeGenerator::generate_all(&output);
        assert!(generated.contains("// === DTO: Agreement ==="));
        assert!(generated.contains("// === Service: Agreement ==="));
        assert!(generated.contains("// === Handler: Agreement ==="));
        assert!(generated.contains("// === DTO: Order ==="));
        assert!(generated.contains("// === Service: Order ==="));
        assert!(generated.contains("// === Handler: Order ==="));
    }
}
