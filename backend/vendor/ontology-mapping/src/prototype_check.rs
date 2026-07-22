//! prototype_check.rs — 原型独立可加载校验（1:1 移植 check-prototype-standalone.py）
//!
//! 9 项检查：外部 CDN / boot-skeleton / React 解构 / try-catch / Babel 解析 /
//! CSS 语法（\:root、::root、大括号、var 引用无定义）/ 外部 CSS :root 合并 /
//! dangerouslySetInnerHTML 冲突 / vendor 路径存在性。
//!
//! Node 委托（项目内部工具链，非外源依赖）：
//! - `bun scripts/parser-utils.mjs find-root-vars|extract-all-refs`（CSS/HTML 解析）
//! - `node -e <babel-check>`（@babel/standalone 离线解析）

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ── 禁用的外部 CDN 模式 ────────────────────────────────

const FORBIDDEN_CDN: &[(&str, &str)] = &[
    (r"https://fonts\.googleapis\.com", "Google Fonts CSS CDN"),
    (r"https://fonts\.gstatic\.com", "Google Fonts gstatic"),
    (r"https://use\.typekit\.net", "Typekit / Adobe Fonts"),
    (
        r"https://cdn\.jsdelivr\.net/npm/mermaid",
        "Mermaid CDN (jsdelivr)",
    ),
    (r"https://unpkg\.com/mermaid", "Mermaid CDN (unpkg)"),
    (
        r"https://cdnjs\.cloudflare\.com/ajax/libs/mermaid",
        "Mermaid CDN (cdnjs)",
    ),
];

const BOOT_SKELETON_INDICATORS: &[&str] = &[
    r#"id="boot-skeleton""#,
    r"\.boot-skeleton\s*\{",
    r"\.boot-loader\s*\{",
];

const BABEL_CHECK_JS: &str = r#"
const filePath = process.argv[1];
if (!filePath) { console.log('NO_FILE'); process.exit(1); }
const fs = require('fs');
const code = fs.readFileSync(filePath, 'utf8');
const re = /<script type="text\/babel"[^>]*>([\s\S]*?)<\/script>/;
const m = code.match(re);
if (!m) { console.log('NO_SCRIPT'); process.exit(0); }
const script = m[1];
try {
  require('@babel/standalone').transform(script, { filename: 'inline', presets: ['react'] });
  console.log('OK');
} catch(e) {
  console.log('ERROR: ' + e.message);
  process.exit(1);
}
"#;

// ── 带超时的子进程执行 ─────────────────────────────────

struct CmdOut {
    stdout: String,
    stderr: String,
    success: bool,
    timed_out: bool,
    spawn_error: Option<String>,
}

fn run_with_timeout(program: &str, args: &[&str], cwd: Option<&Path>, timeout_secs: u64) -> CmdOut {
    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return CmdOut {
                stdout: String::new(),
                stderr: String::new(),
                success: false,
                timed_out: false,
                spawn_error: Some(e.to_string()),
            }
        }
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let out = child.wait_with_output().unwrap();
                return CmdOut {
                    success: out.status.success(),
                    stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    timed_out: false,
                    spawn_error: None,
                };
            }
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return CmdOut {
                        stdout: String::new(),
                        stderr: String::new(),
                        success: false,
                        timed_out: true,
                        spawn_error: None,
                    };
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                return CmdOut {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: false,
                    timed_out: false,
                    spawn_error: Some(e.to_string()),
                }
            }
        }
    }
}

// ── 原型文件收集 ───────────────────────────────────────

pub fn find_prototypes(targets: &[PathBuf]) -> Vec<PathBuf> {
    let name_re = regex::Regex::new(r"(?:[^/]+-)?v\d+\.html$").unwrap();
    let mut files = Vec::new();
    for target in targets {
        if target.is_file() {
            if target.extension().is_some_and(|e| e == "html")
                && name_re.is_match(&target.file_name().unwrap_or_default().to_string_lossy())
                && std::fs::metadata(target)
                    .map(|m| m.len() >= 1024)
                    .unwrap_or(false)
            {
                files.push(target.clone());
            }
        } else if target.is_dir() {
            collect_dir(target, &name_re, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_dir(dir: &Path, name_re: &regex::Regex, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_dir(&p, name_re, out);
        } else if p.extension().is_some_and(|e| e == "html") {
            let s = p.to_string_lossy().to_string();
            if s.contains("/design/fonts/") || s.contains("/design/react.") {
                continue;
            }
            if !name_re.is_match(&p.file_name().unwrap_or_default().to_string_lossy()) {
                continue;
            }
            if std::fs::metadata(&p)
                .map(|m| m.len() >= 1024)
                .unwrap_or(false)
            {
                out.push(p);
            }
        }
    }
}

// ── 各项检查 ──────────────────────────────────────────

fn check_no_external_cdn(content: &str) -> Vec<String> {
    FORBIDDEN_CDN
        .iter()
        .filter(|(pattern, _)| regex::Regex::new(pattern).unwrap().is_match(content))
        .map(|(_, name)| format!("  ✗ 含外部 CDN 引用: {name}"))
        .collect()
}

fn check_boot_skeleton(content: &str) -> Vec<String> {
    let found = BOOT_SKELETON_INDICATORS
        .iter()
        .filter(|p| regex::Regex::new(p).unwrap().is_match(content))
        .count();
    if found == 0 {
        vec!["  ✗ 缺少加载骨架（id='boot-skeleton' 或 .boot-skeleton{} 等）".to_string()]
    } else if found < 2 {
        vec!["  ⚠ 加载骨架不完整（建议同时含 id + CSS 样式）".to_string()]
    } else {
        vec![]
    }
}

fn check_react_destructure(content: &str) -> Vec<String> {
    let destructure = regex::Regex::new(
        r"const\s*\{[^}]*\buseState\b[^}]*\bcreateElement\s*:\s*\w+[^}]*\}\s*=\s*React",
    )
    .unwrap();
    if destructure.is_match(content) {
        return vec![];
    }
    let script_re = |pat: &str| regex::Regex::new(pat).unwrap();
    let has_react_umd =
        script_re(r#"(?i)<script\s+[^>]*\bsrc\s*=\s*["'][^"']*react\.umd\.js["'][^>]*>"#)
            .is_match(content);
    let has_bundle =
        script_re(r#"(?i)<script\s+[^>]*\bsrc\s*=\s*["'][^"']*\.bundle\.js["'][^>]*>"#)
            .is_match(content);
    if has_react_umd && has_bundle {
        return vec![];
    }
    vec![
        "  ✗ 缺少 React 解构声明（inline babel 模式需要 `const { useState, ..., createElement: h } = React;`；ESM build 模式需要 react.umd.js + *.bundle.js 且均未找到）"
            .to_string(),
    ]
}

fn check_render_try_catch(content: &str) -> Vec<String> {
    let re = regex::Regex::new(r"try\s*\{[^}]*root\.render\(").unwrap();
    if content.contains("root.render(h(App))") && !re.is_match(content) {
        return vec![
            "  ⚠ root.render(h(App)) 未包 try/catch（错误时无法设置 document.title）".to_string(),
        ];
    }
    vec![]
}

/// 从 HTML 提取 <style> 块内容列表（纯字符串扫描，与 Python 一致）
fn extract_style_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    while let Some(oi) = content[pos..].find("<style").map(|i| pos + i) {
        let Some(gt) = content[oi..].find('>').map(|i| oi + i) else {
            break;
        };
        let Some(ci) = content[gt..].find("</style>").map(|i| gt + i) else {
            break;
        };
        blocks.push(content[gt + 1..ci].to_string());
        pos = ci + 8;
    }
    blocks
}

/// 调 parser-utils find-root-vars 收集 :root 中的 -- 变量定义
fn collect_root_vars(path: &Path, repo_hint: &Path) -> std::collections::HashSet<String> {
    let mut defs = std::collections::HashSet::new();
    let Some(parser) = find_parser_utils(repo_hint) else {
        return defs;
    };
    let out = run_with_timeout(
        "bun",
        &[
            parser.to_str().unwrap_or(""),
            "find-root-vars",
            path.to_str().unwrap_or(""),
        ],
        None,
        15,
    );
    if out.success && !out.stdout.trim().is_empty() {
        if let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(&out.stdout) {
            for v in list {
                if let Some(prop) = v.get("property").and_then(|p| p.as_str()) {
                    if prop.starts_with("--") {
                        defs.insert(prop.to_string());
                    }
                }
            }
        }
    }
    defs
}

fn check_css_syntax(html_path: &Path, content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let blocks = extract_style_blocks(content);
    let mut all_var_refs = std::collections::HashSet::new();

    let var_re = regex::Regex::new(r"var\(--([\w-]+)").unwrap();
    for (idx, block) in blocks.iter().enumerate() {
        if block.contains("\\:root") {
            issues.push(format!(
                "  ✗ CSS 第 {} 个 <style> 块中出现 \\:root（反斜杠转义冒号），CSS 解析器将其视为元素选择器 <:root> 而非伪类 :root，导致全部设计令牌（--primary/--background 等）不生效",
                idx + 1
            ));
        }
        if block.contains("::root") {
            issues.push(format!(
                "  ✗ CSS 第 {} 个 <style> 块中出现 ::root（应为 :root），导致全部设计令牌（--primary/--background/--muted 等）失效",
                idx + 1
            ));
        }
        // 大括号配平
        let mut depth = 0i32;
        let mut broke = false;
        for ch in block.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
            }
            if depth < 0 {
                issues.push(format!(
                    "  ✗ CSS 第 {} 个 <style> 块中存在多余的 '}}' 闭合（大括号不匹配）",
                    idx + 1
                ));
                broke = true;
                break;
            }
        }
        if !broke && depth != 0 {
            issues.push(format!(
                "  ✗ CSS 第 {} 个 <style> 块中缺少 '}}' 闭合（{depth} 个未闭合的 '{{'）",
                idx + 1
            ));
        }
        // var(--X) 引用

        for cap in var_re.captures_iter(block) {
            all_var_refs.insert(format!("--{}", &cap[1]));
        }
    }

    // :root 定义（inline + 外部 CSS 合并）
    let mut all_var_defs = collect_root_vars(html_path, html_path);

    // 外部 CSS 文件（extract-all-refs → link[href$=.css]）
    if let Some(parser) = find_parser_utils(html_path) {
        let out = run_with_timeout(
            "bun",
            &[
                parser.to_str().unwrap_or(""),
                "extract-all-refs",
                html_path.to_str().unwrap_or(""),
            ],
            None,
            15,
        );
        if out.success && !out.stdout.trim().is_empty() {
            if let Ok(refs) = serde_json::from_str::<Vec<serde_json::Value>>(&out.stdout) {
                for r in refs {
                    if r.get("tag").and_then(|t| t.as_str()) != Some("link") {
                        continue;
                    }
                    let Some(href) = r.get("value").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if !href.ends_with(".css")
                        || href.starts_with("http:")
                        || href.starts_with("https:")
                        || href.starts_with("//")
                        || href.starts_with("data:")
                    {
                        continue;
                    }
                    let css_path = html_path
                        .parent()
                        .map(|p| p.join(href))
                        .and_then(|p| p.canonicalize().ok())
                        .unwrap_or_else(|| PathBuf::from(href));
                    if !css_path.exists() {
                        issues.push(format!(
                            "  ⚠ 外部 CSS 文件不存在: href=\"{href}\"  →  {}",
                            css_path.display()
                        ));
                        continue;
                    }
                    // 包 <style> 写临时文件后 find-root-vars
                    if let Ok(css_content) = std::fs::read_to_string(&css_path) {
                        let tmp = std::env::temp_dir().join(format!(
                            "proto-check-css-{}-{}.html",
                            std::process::id(),
                            all_var_defs.len()
                        ));
                        if std::fs::write(&tmp, format!("<style>{css_content}</style>")).is_ok() {
                            all_var_defs.extend(collect_root_vars(&tmp, html_path));
                            let _ = std::fs::remove_file(&tmp);
                        }
                    }
                }
            }
        }
    }

    if !all_var_refs.is_empty() {
        let missing: Vec<&String> = all_var_refs.difference(&all_var_defs).collect();
        if !missing.is_empty() {
            let mut sorted: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
            sorted.sort();
            issues.push(format!(
                "  ✗ CSS 变量引用 {} 处未在 :root 中定义: {}",
                sorted.len(),
                sorted.join(", ")
            ));
        }
    }
    issues
}

fn check_dangerously_set_inner_html(content: &str) -> Vec<String> {
    let re = regex::Regex::new(
        r#"h\s*\([^)]*?\bdangerouslySetInnerHTML\s*:\s*\{[^}]*\}\s*\}\s*,\s*(?:['\"`])[^'\"`]+(?:['\"`])"#,
    )
    .unwrap();
    re.find_iter(content)
        .map(|m| {
            let snippet: String = m.as_str().chars().take(80).collect();
            format!(
                "  ✗ dangerouslySetInnerHTML 与 text child 冲突: {snippet}（React 禁止同一元素同时使用 dangerouslySetInnerHTML 和 children，须将图标放到子 <span> 元素中: h('tag', null, h('span', {{dangerouslySetInnerHTML: …}}), 'label')"
            )
        })
        .collect()
}

fn check_babel_parse(html_path: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let out = run_with_timeout(
        "node",
        &["-e", BABEL_CHECK_JS, "--", html_path.to_str().unwrap_or("")],
        Some(Path::new("/tmp")), // babel/standalone 安装在 /tmp/node_modules
        30,
    );
    if out.timed_out {
        issues.push("  ⚠ Babel 解析超时（30s）".to_string());
    } else if let Some(e) = out.spawn_error {
        issues.push(format!("  ⚠ Node.js 未安装，跳过 Babel 解析: {e}"));
    } else if !out.success {
        let stderr: String = out.stderr.trim().chars().take(200).collect();
        let stdout: String = out.stdout.trim().chars().take(200).collect();
        if stderr.contains("Cannot find module") {
            issues.push("  ⚠ Babel 解析跳过（@babel/standalone 未安装，运行 `cd /tmp && npm install @babel/standalone`）".to_string());
        } else {
            issues.push(format!(
                "  ✗ Babel 解析失败: {}",
                if stdout.is_empty() { stderr } else { stdout }
            ));
        }
    } else if out.stdout.contains("ERROR:") {
        issues.push(format!("  ✗ Babel 解析失败: {}", out.stdout.trim()));
    }
    issues
}

fn check_vendor_paths(html_path: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let Some(parser) = find_parser_utils(html_path) else {
        return issues;
    };
    let out = run_with_timeout(
        "bun",
        &[
            parser.to_str().unwrap_or(""),
            "extract-all-refs",
            html_path.to_str().unwrap_or(""),
        ],
        None,
        15,
    );
    if !out.success || out.stdout.trim().is_empty() {
        return issues;
    }
    let Ok(refs) = serde_json::from_str::<Vec<serde_json::Value>>(&out.stdout) else {
        return issues;
    };
    for r in refs {
        let Some(val) = r.get("value").and_then(|v| v.as_str()) else {
            continue;
        };
        let attr = r.get("attr").and_then(|v| v.as_str()).unwrap_or("");
        if val.starts_with("http:")
            || val.starts_with("https:")
            || val.starts_with("//")
            || val.starts_with("data:")
            || val.starts_with('/')
            || val.starts_with("about:")
            || val.starts_with('#')
            || val.ends_with(".css")
        // .css 由 check_css_syntax 验证
        {
            continue;
        }
        let resolved = html_path
            .parent()
            .map(|p| p.join(val))
            .and_then(|p| p.canonicalize().ok())
            .unwrap_or_else(|| PathBuf::from(val));
        if !resolved.exists() {
            let (prefix, label) = if attr == "src" {
                ("✗", "vendor 资源")
            } else {
                ("⚠", "本地资源")
            };
            issues.push(format!(
                "  {prefix} {label}不存在: {attr}=\"{val}\"  →  {}",
                resolved.display()
            ));
        }
    }
    issues
}

// ── 单文件与批量入口 ───────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FileReport {
    pub path: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckSummary {
    pub files: usize,
    pub errors: usize,
    pub warnings: usize,
    pub failed_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub results: Vec<FileReport>,
    pub summary: CheckSummary,
}

pub fn check_file(html_path: &Path, run_babel: bool) -> FileReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 统一绝对路径——babel/vendor 解析在 cwd=/tmp 下运行，相对路径会 ENOENT
    let abs_path = html_path
        .canonicalize()
        .unwrap_or_else(|_| html_path.to_path_buf());

    let Ok(content) = std::fs::read_to_string(&abs_path) else {
        return FileReport {
            path: html_path.display().to_string(),
            errors: vec!["  ✗ 读取文件失败".to_string()],
            warnings: vec![],
        };
    };

    errors.extend(check_no_external_cdn(&content));
    for issue in check_boot_skeleton(&content) {
        if issue.contains('✗') {
            errors.push(issue);
        } else {
            warnings.push(issue);
        }
    }
    errors.extend(check_react_destructure(&content));
    for issue in check_css_syntax(&abs_path, &content) {
        if issue.contains('✗') {
            errors.push(issue);
        } else {
            warnings.push(issue);
        }
    }
    errors.extend(check_vendor_paths(&abs_path));
    errors.extend(check_dangerously_set_inner_html(&content));
    warnings.extend(check_render_try_catch(&content));
    if run_babel {
        for issue in check_babel_parse(&abs_path) {
            if issue.contains('✗') {
                errors.push(issue);
            } else {
                warnings.push(issue);
            }
        }
    }

    FileReport {
        path: html_path.display().to_string(),
        errors,
        warnings,
    }
}

/// 批量检查（与 Python main 的退出码语义一致：0 全过 / 1 有错误 / 2 仅警告）
pub fn check_targets(targets: &[PathBuf], run_babel: bool) -> (CheckReport, i32) {
    let prototypes = find_prototypes(targets);
    let mut results = Vec::new();
    let (mut total_errors, mut total_warnings) = (0, 0);
    let mut failed_files = Vec::new();

    for p in &prototypes {
        let report = check_file(p, run_babel);
        total_errors += report.errors.len();
        total_warnings += report.warnings.len();
        if !report.errors.is_empty() {
            failed_files.push(report.path.clone());
        }
        results.push(report);
    }

    let exit = if total_errors > 0 {
        1
    } else if total_warnings > 0 {
        2
    } else {
        0
    };
    (
        CheckReport {
            summary: CheckSummary {
                files: prototypes.len(),
                errors: total_errors,
                warnings: total_warnings,
                failed_files,
            },
            results,
        },
        exit,
    )
}

pub fn find_parser_utils_pub(start: &Path) -> Option<PathBuf> {
    find_parser_utils(start)
}

fn find_parser_utils(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(p) = cur {
        let candidate = p.join("scripts").join("parser-utils.mjs");
        if candidate.exists() {
            return Some(candidate);
        }
        cur = p.parent().map(|x| x.to_path_buf());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdn_detection() {
        assert_eq!(
            check_no_external_cdn("<link href='https://fonts.googleapis.com/css'>").len(),
            1
        );
        assert!(check_no_external_cdn("<style>body{}</style>").is_empty());
    }

    #[test]
    fn test_boot_skeleton() {
        assert!(check_boot_skeleton("<div id=\"boot-skeleton\">").len() == 1); // 仅 1 个指标 → ⚠
        assert!(check_boot_skeleton("<div>").iter().any(|i| i.contains('✗')));
        assert!(check_boot_skeleton("id=\"boot-skeleton\" .boot-skeleton{ }").is_empty());
    }

    #[test]
    fn test_react_destructure_modes() {
        // inline 模式
        assert!(check_react_destructure("const { useState, createElement: h } = React").is_empty());
        // ESM 模式
        assert!(check_react_destructure(
            r#"<script src="vendor/react.umd.js"></script><script src="a-v1.bundle.js"></script>"#
        )
        .is_empty());
        // 都缺 → 报错
        assert_eq!(check_react_destructure("<div>").len(), 1);
    }

    #[test]
    fn test_css_brace_balance() {
        let issues = check_css_syntax(
            Path::new("/nonexistent.html"),
            "<style>:root { --a: 1; }</style>",
        );
        assert!(issues.iter().all(|i| !i.contains("大括号")), "{issues:?}");
        let bad = check_css_syntax(
            Path::new("/nonexistent.html"),
            "<style>:root { --a: 1; </style>",
        );
        assert!(bad.iter().any(|i| i.contains("缺少 '}'")), "{bad:?}");
    }

    #[test]
    fn test_css_root_variants() {
        let bad1 = check_css_syntax(
            Path::new("/nonexistent.html"),
            "<style>\\:root { --a: 1; }</style>",
        );
        assert!(bad1.iter().any(|i| i.contains("\\:root")), "{bad1:?}");
        let bad2 = check_css_syntax(
            Path::new("/nonexistent.html"),
            "<style>::root { --a: 1; }</style>",
        );
        assert!(bad2.iter().any(|i| i.contains("::root")), "{bad2:?}");
    }

    #[test]
    fn test_dangerously_set_inner_html() {
        let bad = check_dangerously_set_inner_html(
            "h('button', {dangerouslySetInnerHTML: {__html: ICON}}, 'label')",
        );
        assert_eq!(bad.len(), 1);
        assert!(check_dangerously_set_inner_html("h('div', null, 'label')").is_empty());
    }

    #[test]
    fn test_extract_style_blocks() {
        let blocks = extract_style_blocks("<style>a{}</style><p>x</p><style>b{}</style>");
        assert_eq!(blocks, vec!["a{}", "b{}"]);
    }
}
