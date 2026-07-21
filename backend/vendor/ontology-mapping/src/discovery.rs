//! discovery.rs — DB-grounded 本体映射发现引擎
//!
//! 1:1 移植自 `scripts/ontology-discover.py`（成熟工具 Rust 化）。
//! 语义保持：台阶式相似度加权、字段覆盖评分、继承链推测、综合评分公式
//! （名称 40% + 字段覆盖 60%，阈值 0.1，TOP-N 排序）。

use anyhow::Result;
use sqlx::PgPool;

use crate::output::{EntityInput, FieldInput, MappingInput};

// ── 继承链根表（按业务域分组）──────────────────────────

/// 单值根表。`product` 族单独用 PRODUCT_ROOTS 表达。
pub const HIERARCHY_ROOTS: &[(&str, &str)] = &[
    ("lifecycle", "zc_id_lifecycle"),
    ("production", "zc_id_production"),
    ("agreement", "zc_id_agreement"),
    ("bill", "zc_id_bill"),
    ("contract", "zc_id_contract"),
    ("inventory", "zc_id_inventory"),
    ("counting", "zc_id_counting"),
    ("evaluation", "zc_id_evaluation"),
    ("detail", "zc_id_detail"),
    ("document", "zc_id_document"),
    ("device", "zc_id_device"),
    ("entity", "zc_id_entity"),
    ("scalar_amount", "zc_id_scal-amount"),
    ("scalar_price", "zc_id_scal-price"),
    ("scalar_date", "zc_id_scal-date"),
    ("scalar_common", "zc_id_scal-common"),
    ("category", "zc_id_category"),
    ("status", "zc_id_status"),
    ("tags", "zc_id_tags"),
    ("level", "zc_id_level"),
    ("unit", "zc_id_unit"),
    ("contacts", "zc_id_contacts"),
    ("appeal", "zc_id_appeal"),
    ("approve", "zc_id_approve"),
    ("subject", "zc_id_subject"),
    ("partner", "zc_id_partner"),
    ("product_purchase", "zc_id_prod-purchase"),
    ("product_all", "zc_id_prod-sales"),
];

/// product 族四分支（Python 中 HIERARCHY_ROOTS["product"] 的 dict 值）
pub const PRODUCT_ROOTS: &[&str] = &[
    "zc_id_prod-request",
    "zc_id_prod-sales",
    "zc_id_prod-made",
    "zc_id_prod-purchase",
];

pub fn hierarchy_root(key: &str) -> Option<&'static str> {
    HIERARCHY_ROOTS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

// ── 字段模式 → 继承链推测 ──────────────────────────────

struct FieldPattern {
    key: &'static str,
    signals: &'static [&'static str],
    weight: f64,
}

const FIELD_PATTERNS: &[FieldPattern] = &[
    FieldPattern {
        key: "lifecycle",
        signals: &["status", "code", "notice", "name", "label"],
        weight: 0.6,
    },
    FieldPattern {
        key: "production",
        signals: &["price", "qty", "quantity", "unit", "product", "sku", "bom"],
        weight: 0.6,
    },
    FieldPattern {
        key: "product_sales",
        signals: &["price", "currency", "amount", "sales", "customer"],
        weight: 0.5,
    },
    FieldPattern {
        key: "agreement",
        signals: &["contract", "term", "clause", "party", "sign"],
        weight: 0.5,
    },
    FieldPattern {
        key: "inventory",
        signals: &["warehouse", "stock", "bin", "location", "onhand", "qty"],
        weight: 0.6,
    },
    FieldPattern {
        key: "evaluation",
        signals: &["score", "rating", "eval", "review", "feedback"],
        weight: 0.5,
    },
    FieldPattern {
        key: "detail",
        signals: &["line_item", "item", "detail", "明细"],
        weight: 0.4,
    },
    FieldPattern {
        key: "document",
        signals: &["file", "attachment", "document", "doc", "blueprint"],
        weight: 0.5,
    },
];

// ── 中文→英文业务术语翻译 ──────────────────────────────

const ZH_TO_EN: &[(&str, &str)] = &[
    ("采购", "purchase"),
    ("订单", "order"),
    ("销售", "sales"),
    ("发票", "invoice"),
    ("合同", "contract"),
    ("协议", "agreement"),
    ("入库", "inbound"),
    ("出库", "outbound"),
    ("库存", "inventory"),
    ("仓库", "warehouse"),
    ("物流", "logistics"),
    ("运输", "transport"),
    ("客户", "customer"),
    ("供应商", "supplier"),
    ("合作伙伴", "partner"),
    ("产品", "product"),
    ("商品", "goods"),
    ("物料", "material"),
    ("价格", "price"),
    ("金额", "amount"),
    ("数量", "qty"),
    ("对账", "reconciliation"),
    ("结算", "settlement"),
    ("付款", "payment"),
    ("收款", "receipt"),
    ("预算", "budget"),
    ("成本", "cost"),
    ("员工", "employee"),
    ("部门", "department"),
    ("组织", "organization"),
    ("项目", "project"),
    ("任务", "task"),
    ("工单", "work_order"),
    ("审批", "approval"),
    ("审核", "review"),
    ("评估", "evaluation"),
    ("质检", "inspection"),
    ("检验", "inspection"),
    ("公告", "notice"),
    ("通知", "notification"),
    ("消息", "message"),
    ("报表", "report"),
    ("看板", "dashboard"),
    ("仪表盘", "dashboard"),
    ("配置", "config"),
    ("设置", "settings"),
    ("权限", "permission"),
    ("用户", "user"),
    ("角色", "role"),
    ("菜单", "menu"),
    ("收货", "receiving"),
    ("发货", "delivery"),
    ("退货", "return"),
    ("交货", "delivery"),
    ("日期", "date"),
    ("状态", "status"),
    ("类型", "type"),
    ("类别", "category"),
    ("名称", "name"),
    ("描述", "description"),
    ("备注", "comment"),
    ("编码", "code"),
    ("编号", "code"),
    ("创建时间", "created_at"),
    ("更新时间", "updated_at"),
    ("创建人", "created_by"),
    ("更新人", "updated_by"),
    ("负责人", "manager"),
    ("联系人", "contact"),
    ("应收", "receivable"),
    ("应付", "payable"),
    ("凭证", "voucher"),
    ("单据", "document"),
    ("请款", "payment_request"),
    ("报价", "quotation"),
    ("询价", "inquiry"),
    ("招标", "bidding"),
    ("索赔", "claim"),
    ("理赔", "claim_settlement"),
    ("保单", "insurance"),
    ("报关", "customs"),
    ("装箱", "packing"),
    ("报关单", "customs_declaration"),
];

pub fn zh_to_en(text: &str) -> String {
    let mut result = text.to_string();
    for (zh, en) in ZH_TO_EN {
        result = result.replace(zh, en);
    }
    result
}

// ── 语义匹配引擎 ───────────────────────────────────────

/// 实体名与表名的语义相似度（0-1），支持中文→英文匹配。
/// 台阶：精确 1.0 → 分词交集 |∩|/|∪|*0.8 → 子串 0.6 → fuzzy ratio*0.5
pub fn name_similarity(entity: &str, table: &str) -> f64 {
    let entity_en = zh_to_en(entity);

    // 去掉前缀（与 Python re.sub 两步入等价）
    let mut clean = table.to_string();
    if let Some(idx) = clean.strip_prefix("zc_id_").and_then(|_| clean.find('-')) {
        // ^zc_id_\w+- 形式：去掉 zc_id_xxx- 前缀
        clean = clean[idx + 1..].to_string();
    } else if let Some(stripped) = clean.strip_prefix("zc_id_") {
        clean = stripped.to_string();
    }
    let clean = clean.replace(['_', '-'], " ").to_lowercase();
    let entity_lower = entity_en.replace(['_', '-'], " ").to_lowercase();

    // 精确匹配
    if entity_lower == clean {
        return 1.0;
    }

    // 分词交集匹配
    let e_words: std::collections::HashSet<&str> = entity_lower.split_whitespace().collect();
    let t_words: std::collections::HashSet<&str> = clean.split_whitespace().collect();
    let intersection = e_words.intersection(&t_words).count();
    if intersection > 0 {
        let union = e_words.union(&t_words).count();
        return intersection as f64 / union as f64 * 0.8;
    }

    // 子串匹配
    if clean.contains(&entity_lower) || entity_lower.contains(&clean) {
        return 0.6;
    }

    // fuzzy 匹配（SequenceMatcher.ratio 等价物）
    similar::TextDiff::from_chars(&entity_lower, &clean).ratio() as f64 * 0.5
}

/// 字段到列的映射规则（与 rules.yaml / Python field_to_col 一致）
const FIELD_TO_COL: &[(&str, &str)] = &[
    ("name", "notice"),
    ("title", "notice"),
    ("label", "notice"),
    ("desc", "notice"),
    ("description", "notice"),
    ("code", "code"),
    ("sku", "code"),
    ("status", "fk_status"),
    ("flag", "flag"),
    ("type", "_t_"),
    ("category", "ck_category"),
    ("tags", "tk_tags"),
    ("amount", "qk_amount"),
    ("price", "qk_price"),
    ("qty", "qk_qty"),
    ("quantity", "qk_qty"),
    ("date", "qk_date"),
    ("created_at", "created_at"),
    ("id", "id"),
    ("enable", "enable"),
    ("currency", "sk_currency"),
    ("unit", "sk_unit"),
    ("version", "sk_version"),
];

/// 字段覆盖度评分——实体的字段在表中有多少列可匹配。
pub fn field_coverage_from_cols(
    entity_fields: &[String],
    table_cols: &std::collections::HashSet<String>,
) -> f64 {
    if entity_fields.is_empty() || table_cols.is_empty() {
        return 0.0;
    }
    let mut matched = 0usize;
    for field in entity_fields {
        let f_lower = field.to_lowercase();
        // 直接匹配列名
        if table_cols.contains(&f_lower) {
            matched += 1;
            continue;
        }
        // 通过映射表匹配
        if let Some((_, target)) = FIELD_TO_COL.iter().find(|(k, _)| *k == f_lower) {
            if table_cols.contains(*target) {
                matched += 1;
                continue;
            }
        }
        // 前缀匹配（fk_*, qk_*, sk_* 等）
        if (f_lower.starts_with("fk_")
            || f_lower.starts_with("qk_")
            || f_lower.starts_with("sk_")
            || f_lower.starts_with("ck_")
            || f_lower.starts_with("tk_")
            || f_lower.starts_with("lk_"))
            && table_cols.contains(&f_lower)
        {
            matched += 1;
        }
    }
    matched as f64 / entity_fields.len() as f64
}

/// 根据实体名和字段推测应查哪个继承链根表。
pub fn infer_hierarchy(entity_name: &str, fields: &[String]) -> &'static str {
    let entity_en = zh_to_en(entity_name);
    let name_lower = entity_en.to_lowercase();
    let all_text = format!("{} {}", name_lower, fields.join(" ").to_lowercase());

    let mut best_key = "lifecycle";
    let mut best_score = 0.0f64;
    for pat in FIELD_PATTERNS {
        let mut score = 0.0;
        for signal in pat.signals {
            if all_text.contains(signal) {
                score += pat.weight;
            }
        }
        for signal in pat.signals {
            if name_lower.contains(signal) {
                score += pat.weight * 0.5;
            }
        }
        if score > best_score {
            best_score = score;
            best_key = pat.key;
        }
    }

    let mapped = match best_key {
        "lifecycle" => "lifecycle",
        "production" => "production",
        "product_sales" => "product",
        "agreement" => "agreement",
        "inventory" => "inventory",
        "evaluation" => "evaluation",
        "detail" => "detail",
        "document" => "document",
        _ => "lifecycle",
    };

    if mapped == "product" {
        if all_text.contains("purchase") || all_text.contains("buy") {
            return "product_purchase";
        }
        return "product";
    }
    mapped
}

// ── DB 查询（sqlx 直连，替代 Python 的 mise run schema-info 链路）──

/// 调用 isahl_meta.gf_query_leafs 获取叶表清单。
pub async fn query_leafs(pool: &PgPool, parent: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT unnest(isahl_meta.gf_query_leafs($1)) AS t")
        .bind(parent)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// 查询表的列名集合（information_schema，ordinal_position <= 60）。
pub async fn query_columns(
    pool: &PgPool,
    table: &str,
) -> Result<std::collections::HashSet<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT column_name FROM information_schema.columns \
         WHERE table_schema = 'isahl' AND table_name = $1 AND ordinal_position <= 60",
    )
    .bind(table)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// 查询表的注释。
pub async fn query_table_comment(pool: &PgPool, table: &str) -> Result<Option<String>> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT pgd.description FROM pg_catalog.pg_class pc \
         LEFT JOIN pg_catalog.pg_description pgd ON pgd.objoid = pc.oid AND pgd.objsubid = 0 \
         WHERE pc.relname = $1 \
           AND pc.relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'isahl')",
    )
    .bind(table)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.0))
}

/// 字段覆盖度评分（DB 版：查列后走纯函数评分）。
pub async fn field_coverage(pool: &PgPool, entity_fields: &[String], table: &str) -> f64 {
    match query_columns(pool, table).await {
        Ok(cols) => field_coverage_from_cols(entity_fields, &cols),
        Err(_) => 0.0,
    }
}

// ── 候选表匹配 ─────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct TableCandidate {
    pub table: String,
    pub score: f64,
    pub name_score: f64,
    pub field_score: f64,
    pub comment: Option<String>,
}

/// 按字段覆盖度 + 表名匹配综合评分（名称 40% + 字段 60%，阈值 0.1）。
pub async fn match_tables(
    pool: &PgPool,
    entity: &str,
    fields: &[String],
    parent: Option<&str>,
    top_n: usize,
) -> Result<Vec<TableCandidate>> {
    let leafs = match parent {
        Some(p) => query_leafs(pool, p).await?,
        None => {
            let hint = infer_hierarchy(entity, fields);
            if hint == "product" || hint == "product_purchase" || hint == "product_all" {
                // product 族多分支（Python: isinstance(root, dict) → 全分支合并）
                let mut all = Vec::new();
                for root in PRODUCT_ROOTS {
                    all.extend(query_leafs(pool, root).await?);
                }
                all
            } else {
                let root = hierarchy_root(hint).unwrap_or("zc_id_lifecycle");
                query_leafs(pool, root).await?
            }
        }
    };
    if leafs.is_empty() {
        return Ok(vec![]);
    }

    let mut scored: Vec<TableCandidate> = Vec::new();
    for table in leafs {
        let name_score = name_similarity(entity, &table);
        let field_score = if fields.is_empty() {
            0.0
        } else {
            field_coverage(pool, fields, &table).await
        };
        let combined = name_score * 0.4 + field_score * 0.6;
        if combined > 0.1 {
            scored.push(TableCandidate {
                table,
                score: combined,
                name_score,
                field_score,
                comment: None,
            });
        }
    }
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_n);

    // 附注释（仅 TOP-N，减少 DB 往返）
    for cand in &mut scored {
        cand.comment = query_table_comment(pool, &cand.table).await.ok().flatten();
    }
    Ok(scored)
}

/// 按实体名搜索候选叶表（仅名称相似度，阈值 0.15）。
pub async fn search_tables(
    pool: &PgPool,
    entity: &str,
    parent: &str,
    top_n: usize,
) -> Result<Vec<TableCandidate>> {
    let leafs = query_leafs(pool, parent).await?;
    let mut scored: Vec<TableCandidate> = leafs
        .into_iter()
        .map(|t| {
            let s = name_similarity(entity, &t);
            (t, s)
        })
        .filter(|(_, s)| *s > 0.15)
        .map(|(table, s)| TableCandidate {
            table,
            score: s,
            name_score: s,
            field_score: 0.0,
            comment: None,
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_n);
    for cand in &mut scored {
        cand.comment = query_table_comment(pool, &cand.table).await.ok().flatten();
    }
    Ok(scored)
}

/// 分层快速定位——先推测父表层级，再按名称搜索。
pub async fn locate_tables(
    pool: &PgPool,
    entity: &str,
    hint: Option<&str>,
    fields: &[String],
    top_n: usize,
) -> Result<Vec<TableCandidate>> {
    let inferred = infer_hierarchy(entity, fields);
    let root_key = hint.unwrap_or(inferred);
    let leafs = if root_key == "product" {
        let mut all = Vec::new();
        for root in PRODUCT_ROOTS {
            all.extend(query_leafs(pool, root).await?);
        }
        all
    } else {
        let root = hierarchy_root(root_key).unwrap_or("zc_id_lifecycle");
        query_leafs(pool, root).await?
    };
    let mut scored: Vec<TableCandidate> = leafs
        .into_iter()
        .map(|t| {
            let s = name_similarity(entity, &t);
            (t, s)
        })
        .filter(|(_, s)| *s > 0.15)
        .map(|(table, s)| TableCandidate {
            table,
            score: s,
            name_score: s,
            field_score: 0.0,
            comment: None,
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_n);
    for cand in &mut scored {
        cand.comment = query_table_comment(pool, &cand.table).await.ok().flatten();
    }
    Ok(scored)
}

// ── MappingInput 生成 ─────────────────────────────────

/// 将发现的表名 + 实体结构输出为 MappingInput（与 Python to-mapping-input 一致）。
/// 字段名经 zh_to_en 翻译；amount/total/price/qty/quantity 归为 number，其余 string。
pub fn to_mapping_input(
    entity: &str,
    table: &str,
    fields: &[String],
    scene_code: &str,
    factor_ids: &[String],
) -> MappingInput {
    let entity_en = zh_to_en(entity);
    let field_list: Vec<FieldInput> = fields
        .iter()
        .map(|f| zh_to_en(f))
        .filter(|f_en| f_en.to_lowercase() != "id")
        .map(|f_en| {
            let f_lower = f_en.to_lowercase();
            let ftype = match f_lower.as_str() {
                "amount" | "total" | "price" | "qty" | "quantity" => "number",
                _ => "string",
            };
            FieldInput {
                name: f_en,
                field_type: ftype.to_string(),
                format: None,
                r#enum: vec![],
            }
        })
        .collect();

    MappingInput {
        scene_code: scene_code.to_string(),
        factor_ids: factor_ids.to_vec(),
        entities: vec![EntityInput {
            name: entity_en,
            table: Some(table.to_string()),
            fields: field_list,
            nested: vec![],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_zh_to_en() {
        // 与 Python dict.replace 一致：无空格拼接、未收录词原样保留
        assert_eq!(zh_to_en("采购订单"), "purchaseorder");
        assert_eq!(zh_to_en("库存"), "inventory");
        assert_eq!(zh_to_en("Order"), "Order"); // 英文原样
    }

    #[test]
    fn test_name_similarity_exact() {
        // 精确档：entity_lower == clean（zc_id_xxx- 前缀已剥离）
        assert_eq!(name_similarity("purchase", "zc_id_proc-purchase"), 1.0);
    }

    #[test]
    fn test_name_similarity_word_intersection() {
        // 分词交集：{purchase,order}∩{purchase} / {purchase,order} = 1/2 * 0.8
        let s = name_similarity("purchase order", "zc_id_appr-purchase");
        assert!((s - 0.4).abs() < 1e-9, "score={s}");
    }

    #[test]
    fn test_name_similarity_substring() {
        // 子串档：无共同分词，但 clean 是 entity_lower 的子串
        assert_eq!(name_similarity("xyland", "zc_id_orde-land"), 0.6);
    }

    #[test]
    fn test_name_similarity_fuzzy_below_substring() {
        let s = name_similarity("xyzzy", "zc_id_orde-land");
        assert!(s < 0.6, "fuzzy 应低于子串档, got {s}");
    }

    #[test]
    fn test_field_coverage_direct_and_mapped() {
        let cols: std::collections::HashSet<String> = ["notice", "code", "qk_amount", "fk_status"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let f = fields(&["name", "amount", "status", "nonexistent"]);
        let score = field_coverage_from_cols(&f, &cols);
        assert!((score - 0.75).abs() < 1e-9, "score={score}");
    }

    #[test]
    fn test_field_coverage_prefixed_passthrough() {
        let cols: std::collections::HashSet<String> =
            ["qk_custom"].iter().map(|s| s.to_string()).collect();
        let f = fields(&["qk_custom"]);
        assert!((field_coverage_from_cols(&f, &cols) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_infer_hierarchy_lifecycle_default() {
        assert_eq!(
            infer_hierarchy("xyzzy_unknown", &fields(&["foo"])),
            "lifecycle"
        );
    }

    #[test]
    fn test_infer_hierarchy_inventory() {
        assert_eq!(
            infer_hierarchy("stock keeping", &fields(&["warehouse", "qty"])),
            "inventory"
        );
    }

    #[test]
    fn test_infer_hierarchy_product_purchase_branch() {
        // product_sales 信号组占优且含 purchase → product_purchase 分支
        assert_eq!(
            infer_hierarchy("purchase sales", &fields(&["currency", "amount"])),
            "product_purchase"
        );
    }

    #[test]
    fn test_to_mapping_input_translates_and_types() {
        let input = to_mapping_input(
            "采购订单",
            "zc_id_proc-purchase",
            &fields(&["金额", "date", "id"]),
            "",
            &[],
        );
        assert_eq!(input.scene_code, "");
        assert!(input.factor_ids.is_empty());
        let e = &input.entities[0];
        assert_eq!(e.name, "purchaseorder"); // Python dict.replace 无空格拼接
        assert_eq!(e.table.as_deref(), Some("zc_id_proc-purchase"));
        // id 被跳过；金额→number；date→string
        assert_eq!(e.fields.len(), 2);
        assert_eq!(e.fields[0].name, "amount");
        assert_eq!(e.fields[0].field_type, "number");
        assert_eq!(e.fields[1].field_type, "string");
    }
}
