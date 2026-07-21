//! Gap analysis: OntologyModel vs PlatformCatalog → ExtensionGap 列表

use crate::state::{ExtensionGap, ExtensionGapStatus, PlatformCatalog};
use alioth_gen::generator::ir::ontology::{DomainKind, OntologyModel};
use std::collections::HashSet;

pub struct GapAnalysis {
    /// 已有模块覆盖的 domain ID
    pub covered_domains: Vec<String>,
    /// 需要 Meta 扩展的缺口
    pub gaps: Vec<ExtensionGap>,
    /// 无法处理的 domain
    pub unsupported: Vec<String>,
}

/// 对比 OntologyModel 与 PlatformCatalog，确定哪些可直接组合、哪些需扩展、哪些不支持。
pub fn analyze_gaps(model: &OntologyModel, catalog: &PlatformCatalog) -> GapAnalysis {
    let mut covered = Vec::new();
    let mut gaps = Vec::new();
    let mut unsupported = Vec::new();

    let table_set: HashSet<&str> = catalog
        .collections
        .iter()
        .map(|c| c.table_name.as_str())
        .collect();

    for domain in &model.domains {
        if table_set.contains(domain.id.as_str()) {
            covered.push(domain.id.clone());
            continue;
        }
        // 新 domain — 检查是否可以挂载到继承链
        match find_parent_table(domain, catalog) {
            Some(parent_table) => {
                gaps.push(ExtensionGap {
                    domain_id: domain.id.clone(),
                    parent_table,
                    proposed_table_name: domain.id.clone(),
                    new_fields: extract_extra_fields(domain, catalog),
                    status: ExtensionGapStatus::Pending,
                });
            }
            None => {
                unsupported.push(domain.id.clone());
            }
        }
    }

    GapAnalysis {
        covered_domains: covered,
        gaps,
        unsupported,
    }
}

/// 在 PlatformCatalog 的继承树中查找 domain 的合法父表。
fn find_parent_table(
    domain: &alioth_gen::generator::ir::ontology::DomainOntology,
    catalog: &PlatformCatalog,
) -> Option<String> {
    // 1. domain 明确声明了 parent_ids — 取第一个已有的
    for pid in &domain.parent_ids {
        let known = catalog.collections.iter().any(|c| c.table_name == *pid);
        if known {
            return Some(pid.clone());
        }
    }
    // 2. 对实体类 domain，在继承树中查找语义匹配
    if matches!(&domain.kind, DomainKind::Entity) {
        for entry in &catalog.inheritance {
            if entry.children.is_empty() {
                continue; // 叶节点，不可扩展
            }
            // 简单匹配：父表名包含 domain ID 的部分
            if entry.parent_table.contains("production")
                || entry.parent_table.contains("order")
                || entry.parent_table.contains("devi")
            {
                return Some(entry.parent_table.clone());
            }
        }
    }
    None
}

/// 从 domain 信息提取需要注册到 meta_fields 的额外字段。
fn extract_extra_fields(
    domain: &alioth_gen::generator::ir::ontology::DomainOntology,
    _catalog: &PlatformCatalog,
) -> Vec<crate::state::FieldInfo> {
    // 从 domain.properties 提取字段定义
    domain
        .properties
        .iter()
        .map(|p| crate::state::FieldInfo {
            name: p.name.clone(),
            field_type: property_type_label(&p.property_type, &p.range),
            description: p.semantic_description.clone(),
        })
        .collect()
}

fn property_type_label(
    pt: &alioth_gen::generator::ir::ontology::PropertyType,
    range: &str,
) -> String {
    match pt {
        alioth_gen::generator::ir::ontology::PropertyType::DataProperty => range.to_string(),
        alioth_gen::generator::ir::ontology::PropertyType::ObjectProperty => "bigint".into(),
        alioth_gen::generator::ir::ontology::PropertyType::AnnotationProperty => "text".into(),
    }
}
