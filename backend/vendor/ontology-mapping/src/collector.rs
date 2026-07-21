use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceJson {
    pub id: String,
    #[serde(default)]
    pub ontology: Option<FactorOntology>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FactorOntology {
    #[serde(default)]
    pub entities: Vec<FactorEntityMapping>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FactorEntityMapping {
    pub name: String,
    pub table: String,
    #[serde(default)]
    pub inherits: Option<String>,
    /// 本体坐标。允许缺失（历史产物存在 coords=null 或键漂移），
    /// 缺失时实体仍被收集（表匹配不依赖坐标），由调用方决定如何处理。
    #[serde(default)]
    pub coordinates: Option<FactorCoordinates>,
    #[serde(default)]
    pub field_mappings: Vec<FactorFieldMapping>,
    #[serde(default)]
    pub relationships: Vec<FactorRelationship>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FactorCoordinates {
    /// 场景码。历史产物曾使用 `block` 键（键漂移），通过 alias 兼容。
    #[serde(alias = "block")]
    pub scene: String,
    pub factor: String,
    pub function: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FactorFieldMapping {
    pub json_path: String,
    pub column: String,
    #[serde(default)]
    pub scalar: Option<String>,
    #[serde(default)]
    pub ref_table: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FactorRelationship {
    pub target: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(default)]
    pub via: Option<String>,
}

pub fn collect_service_mappings(
    services_dir: impl AsRef<Path>,
) -> Result<Vec<FactorEntityMapping>> {
    let mut mappings = Vec::new();
    let dir = std::fs::read_dir(services_dir)?;

    for entry in dir {
        let entry = entry?;
        let service_json_path = entry.path().join("service.json");
        if service_json_path.exists() {
            let content = std::fs::read_to_string(&service_json_path)?;
            match serde_json::from_str::<ServiceJson>(&content) {
                Ok(service) => {
                    if let Some(ontology) = service.ontology {
                        for entity in ontology.entities {
                            if entity.coordinates.is_none() {
                                eprintln!(
                                    "[ontology-mapping] WARN: {} 实体 `{}` 缺少有效 coordinates（仍收集，表匹配不受影响）",
                                    service_json_path.display(),
                                    entity.name
                                );
                            }
                            mappings.push(entity);
                        }
                    }
                }
                Err(e) => {
                    // 不再静默跳过——反序列化失败必须可见
                    eprintln!(
                        "[ontology-mapping] ERROR: 解析 {} 失败，该 service 的实体全部跳过: {e}",
                        service_json_path.display()
                    );
                }
            }
        }
    }

    Ok(mappings)
}
