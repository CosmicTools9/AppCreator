//! Quality Report Generator
//!
//! 生成数据质量报告，支持多种格式：
//! - JSON: 机器可读格式
//! - HTML: 可视化报告
//! - Markdown: 文档格式

use crate::generator::ir::quality::OntologyQualityMetrics;
use crate::generator::ir::GeneratorModel;
use crate::generator::GenerateError;

/// Generate JSON format quality report
pub fn generate_json_report(
    model: &GeneratorModel,
    metrics: &OntologyQualityMetrics,
) -> Result<String, GenerateError> {
    let report = FullQualityReport::from_model(model, metrics);

    serde_json::to_string_pretty(&report)
        .map_err(|e| GenerateError::Validation(format!("JSON serialization error: {}", e)))
}

/// Generate HTML format quality report
pub fn generate_html_report(
    model: &GeneratorModel,
    metrics: &OntologyQualityMetrics,
) -> Result<String, GenerateError> {
    let report = FullQualityReport::from_model(model, metrics);

    let mut html = String::new();

    // HTML header
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"zh-CN\">\n");
    html.push_str("<head>\n");
    html.push_str("    <meta charset=\"UTF-8\">\n");
    html.push_str(
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
    );
    html.push_str("    <title>数据质量报告</title>\n");
    html.push_str("    <style>\n");
    html.push_str(include_str!("report_styles.css"));
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");

    // Report header
    html.push_str("    <div class=\"container\">\n");
    html.push_str("        <header>\n");
    html.push_str("            <h1>数据质量报告</h1>\n");
    html.push_str(&format!(
        "            <p class=\"timestamp\">生成时间: {}</p>\n",
        report.generated_at
    ));
    html.push_str(&format!(
        "            <p class=\"model-info\">实体数量: {}</p>\n",
        model.entities.len()
    ));
    html.push_str("        </header>\n");

    // Overall score section
    html.push_str("        <section class=\"overall-score\">\n");
    html.push_str(&format!(
        "            <div class=\"score-card grade-{grade}\">\n",
        grade = calculate_grade(metrics.overall_score()).to_lowercase()
    ));
    html.push_str("                <h2>总体质量评分</h2>\n");
    html.push_str(&format!(
        "                <div class=\"score-value\">{:.1}%</div>\n",
        metrics.overall_score() * 100.0
    ));
    html.push_str(&format!(
        "                <div class=\"score-grade\">等级: {}</div>\n",
        calculate_grade(metrics.overall_score())
    ));
    html.push_str("            </div>\n");
    html.push_str("        </section>\n");

    // Ontology metrics section
    html.push_str("        <section class=\"ontology-metrics\">\n");
    html.push_str("            <h2>本体质量指标</h2>\n");
    html.push_str("            <div class=\"metrics-grid\">\n");

    html.push_str(&format!(
        r#"                <div class="metric-card">
                    <div class="metric-label">文档覆盖率</div>
                    <div class="metric-value">{:.1}%</div>
                    <div class="metric-bar"><div class="metric-fill" style="width: {:.1}%"></div></div>
                </div>
"#,
        metrics.documentation_coverage * 100.0,
        metrics.documentation_coverage * 100.0
    ));

    html.push_str(&format!(
        r#"                <div class="metric-card">
                    <div class="metric-label">约束覆盖率</div>
                    <div class="metric-value">{:.1}%</div>
                    <div class="metric-bar"><div class="metric-fill" style="width: {:.1}%"></div></div>
                </div>
"#,
        metrics.constraint_coverage * 100.0,
        metrics.constraint_coverage * 100.0
    ));

    html.push_str(&format!(
        r#"                <div class="metric-card">
                    <div class="metric-label">质量规则覆盖率</div>
                    <div class="metric-value">{:.1}%</div>
                    <div class="metric-bar"><div class="metric-fill" style="width: {:.1}%"></div></div>
                </div>
"#,
        metrics.quality_rule_coverage * 100.0,
        metrics.quality_rule_coverage * 100.0
    ));

    html.push_str(&format!(
        r#"                <div class="metric-card">
                    <div class="metric-label">最大继承深度</div>
                    <div class="metric-value">{}</div>
                </div>
"#,
        metrics.max_hierarchy_depth
    ));

    html.push_str("            </div>\n");
    html.push_str("        </section>\n");

    // Entity quality section
    html.push_str("        <section class=\"entity-quality\">\n");
    html.push_str("            <h2>实体质量详情</h2>\n");
    html.push_str("            <table class=\"data-table\">\n");
    html.push_str("                <thead>\n");
    html.push_str("                    <tr>\n");
    html.push_str("                        <th>实体名称</th>\n");
    html.push_str("                        <th>质量规则数</th>\n");
    html.push_str("                        <th>字段质量规则数</th>\n");
    html.push_str("                        <th>状态</th>\n");
    html.push_str("                    </tr>\n");
    html.push_str("                </thead>\n");
    html.push_str("                <tbody>\n");

    for entity in &model.entities {
        let entity_quality_count = entity.quality_rules.len();
        let field_quality_count: usize = entity.fields.iter().map(|f| f.quality_rules.len()).sum();

        let status = if entity_quality_count > 0 || field_quality_count > 0 {
            "<span class=\"badge badge-success\">已配置</span>"
        } else {
            "<span class=\"badge badge-warning\">未配置</span>"
        };

        html.push_str(&format!(
            r#"                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                    </tr>
"#,
            entity.name.pascal, entity_quality_count, field_quality_count, status
        ));
    }

    html.push_str("                </tbody>\n");
    html.push_str("            </table>\n");
    html.push_str("        </section>\n");

    // Footer
    html.push_str("        <footer>\n");
    html.push_str("            <p>由 AliothStudio 自动生成</p>\n");
    html.push_str("        </footer>\n");
    html.push_str("    </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    Ok(html)
}

/// Generate Markdown format quality report
pub fn generate_markdown_report(
    model: &GeneratorModel,
    metrics: &OntologyQualityMetrics,
) -> Result<String, GenerateError> {
    let mut md = String::new();

    // Title
    md.push_str("# 数据质量报告\n\n");

    // Metadata
    md.push_str(&format!(
        "**生成时间**: {}\n\n",
        chrono::Utc::now().to_rfc3339()
    ));
    md.push_str(&format!("**实体数量**: {}\n\n", model.entities.len()));
    md.push_str(&format!("**枚举数量**: {}\n\n", model.enums.len()));

    // Overall score
    md.push_str("## 总体质量评分\n\n");
    md.push_str(&format!(
        "**评分**: {:.1}%\n\n",
        metrics.overall_score() * 100.0
    ));
    md.push_str(&format!(
        "**等级**: {}\n\n",
        calculate_grade(metrics.overall_score())
    ));

    // Ontology metrics
    md.push_str("## 本体质量指标\n\n");
    md.push_str("| 指标 | 值 |\n");
    md.push_str("|------|-----|\n");
    md.push_str(&format!(
        "| 文档覆盖率 | {:.1}% |\n",
        metrics.documentation_coverage * 100.0
    ));
    md.push_str(&format!(
        "| 约束覆盖率 | {:.1}% |\n",
        metrics.constraint_coverage * 100.0
    ));
    md.push_str(&format!(
        "| 质量规则覆盖率 | {:.1}% |\n",
        metrics.quality_rule_coverage * 100.0
    ));
    md.push_str(&format!(
        "| 最大继承深度 | {} |\n",
        metrics.max_hierarchy_depth
    ));
    md.push_str(&format!(
        "| 平均继承深度 | {:.1} |\n",
        metrics.avg_hierarchy_depth
    ));
    md.push_str(&format!(
        "| 无属性类数量 | {} |\n",
        metrics.classes_without_properties
    ));
    md.push_str(&format!(
        "| 无约束属性数量 | {} |\n\n",
        metrics.properties_without_constraints
    ));

    // Entity details
    md.push_str("## 实体质量详情\n\n");

    for entity in &model.entities {
        md.push_str(&format!("### {}\n\n", entity.name.pascal));

        if let Some(desc) = &entity.description {
            md.push_str(&format!("{}\n\n", desc));
        }

        // Entity-level quality rules
        if !entity.quality_rules.is_empty() {
            md.push_str("**实体级质量规则**:\n\n");
            for rule in &entity.quality_rules {
                md.push_str(&format!(
                    "- **{:?}**: 阈值 {:.0}%\n",
                    rule.metric,
                    rule.threshold * 100.0
                ));
            }
            md.push('\n');
        }

        // Field-level quality rules
        let fields_with_rules: Vec<_> = entity
            .fields
            .iter()
            .filter(|f| !f.quality_rules.is_empty())
            .collect();

        if !fields_with_rules.is_empty() {
            md.push_str("**字段级质量规则**:\n\n");
            md.push_str("| 字段 | 指标 | 阈值 |\n");
            md.push_str("|------|------|------|\n");

            for field in &fields_with_rules {
                for rule in &field.quality_rules {
                    md.push_str(&format!(
                        "| {} | {:?} | {:.0}% |\n",
                        field.name.raw,
                        rule.metric,
                        rule.threshold * 100.0
                    ));
                }
            }
            md.push('\n');
        }

        if entity.quality_rules.is_empty() && fields_with_rules.is_empty() {
            md.push_str("*未配置质量规则*\n\n");
        }
    }

    // Recommendations
    md.push_str("## 改进建议\n\n");

    if metrics.documentation_coverage < 0.8 {
        md.push_str("1. **提高文档覆盖率**: 为更多实体和字段添加描述\n");
    }
    if metrics.constraint_coverage < 0.8 {
        md.push_str("2. **增加约束定义**: 为字段添加更多验证约束\n");
    }
    if metrics.quality_rule_coverage < 0.5 {
        md.push_str("3. **添加质量规则**: 为关键实体配置数据质量验证\n");
    }
    if metrics.classes_without_properties > 0 {
        md.push_str("4. **完善类定义**: 为无属性的类添加适当的字段\n");
    }

    md.push_str("\n---\n\n");
    md.push_str("*由 AliothStudio 自动生成*\n");

    Ok(md)
}

/// Calculate grade from score
fn calculate_grade(score: f64) -> &'static str {
    match score {
        s if s >= 0.95 => "A",
        s if s >= 0.90 => "B",
        s if s >= 0.80 => "C",
        s if s >= 0.70 => "D",
        _ => "F",
    }
}

/// Full quality report structure for JSON serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FullQualityReport {
    generated_at: String,
    overall_score: f64,
    grade: String,
    ontology_metrics: OntologyQualityMetrics,
    entity_reports: Vec<EntityQualitySummary>,
}

impl FullQualityReport {
    fn from_model(model: &GeneratorModel, metrics: &OntologyQualityMetrics) -> Self {
        let entity_reports: Vec<_> = model
            .entities
            .iter()
            .map(EntityQualitySummary::from_entity)
            .collect();

        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            overall_score: metrics.overall_score(),
            grade: calculate_grade(metrics.overall_score()).to_string(),
            ontology_metrics: metrics.clone(),
            entity_reports,
        }
    }
}

/// Entity quality summary for reports
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EntityQualitySummary {
    name: String,
    quality_rule_count: usize,
    field_quality_rule_count: usize,
    has_quality_config: bool,
}

impl EntityQualitySummary {
    fn from_entity(entity: &crate::generator::ir::GeneratorEntity) -> Self {
        let field_quality_count: usize = entity.fields.iter().map(|f| f.quality_rules.len()).sum();

        Self {
            name: entity.name.raw.clone(),
            quality_rule_count: entity.quality_rules.len(),
            field_quality_rule_count: field_quality_count,
            has_quality_config: !entity.quality_rules.is_empty() || field_quality_count > 0,
        }
    }
}

// CSS styles for HTML report (embedded)
#[allow(dead_code)]
const REPORT_STYLES: &str = r#"
:root {
    --primary-color: #3b82f6;
    --success-color: #10b981;
    --warning-color: #f59e0b;
    --error-color: #ef4444;
    --bg-color: #f3f4f6;
    --card-bg: #ffffff;
    --text-color: #1f2937;
    --text-muted: #6b7280;
    --border-color: #e5e7eb;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: var(--bg-color);
    color: var(--text-color);
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

header {
    text-align: center;
    margin-bottom: 2rem;
}

header h1 {
    font-size: 2rem;
    margin-bottom: 0.5rem;
}

header .timestamp {
    color: var(--text-muted);
}

.score-card {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.score-card.grade-a { border-top: 4px solid var(--success-color); }
.score-card.grade-b { border-top: 4px solid #84cc16; }
.score-card.grade-c { border-top: 4px solid var(--warning-color); }
.score-card.grade-d { border-top: 4px solid #f97316; }
.score-card.grade-f { border-top: 4px solid var(--error-color); }

.score-value {
    font-size: 3rem;
    font-weight: bold;
    margin: 1rem 0;
}

.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
}

.metric-card {
    background: var(--card-bg);
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.metric-label {
    color: var(--text-muted);
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
}

.metric-value {
    font-size: 1.5rem;
    font-weight: bold;
}

.metric-bar {
    height: 4px;
    background: var(--border-color);
    border-radius: 2px;
    margin-top: 0.5rem;
    overflow: hidden;
}

.metric-fill {
    height: 100%;
    background: var(--primary-color);
    transition: width 0.3s ease;
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 1rem;
    background: var(--card-bg);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.data-table th,
.data-table td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
}

.data-table th {
    background: #f9fafb;
    font-weight: 600;
}

.badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
}

.badge-success {
    background: #d1fae5;
    color: #065f46;
}

.badge-warning {
    background: #fef3c7;
    color: #92400e;
}

section {
    margin-bottom: 2rem;
}

section h2 {
    margin-bottom: 1rem;
    font-size: 1.5rem;
}

footer {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
    border-top: 1px solid var(--border-color);
    margin-top: 2rem;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_grade() {
        assert_eq!(calculate_grade(0.96), "A");
        assert_eq!(calculate_grade(0.92), "B");
        assert_eq!(calculate_grade(0.85), "C");
        assert_eq!(calculate_grade(0.75), "D");
        assert_eq!(calculate_grade(0.60), "F");
    }

    #[test]
    fn test_generate_markdown_report() {
        let model = GeneratorModel {
            i18n_config: None,
            entities: vec![],
            enums: vec![],
            metadata: Default::default(),
            exceptions: vec![],
            exception_handlers: vec![],
            external_dependencies: vec![],
        };

        let metrics = OntologyQualityMetrics::default();
        let report = generate_markdown_report(&model, &metrics).unwrap();

        assert!(report.contains("数据质量报告"));
        assert!(report.contains("总体质量评分"));
        assert!(report.contains("本体质量指标"));
    }
}
