//! ESM Runner — 调用 prototype-tool.js + sync-prototype.sh 的子进程封装。
//!
//! 替换旧的 CDN babel `write_app_prototype`(已删除)。
//!
//! 流程:
//! 1. `generate_and_build_app_tsx`(staging 阶段):生成 llm-tsx/app.tsx → 复制到
//!    Pre-Proc/{ns}/Prototypes/Apps/{code}/ → 调用 `bun prototype-tool.js build`
//!    - 内建 P3 ESM test-driven 回流:build 失败 → 捕获 esbuild/standalone 错误
//!      → 回灌 `render_app_tsx_template` 上下文 → 重渲 app.tsx → 重 build,
//!      最多 `evaluate::MAX_EVAL_ITERATIONS`(3) 次,收敛即停。
//! 2. `sync_prototype`(build_app 阶段,final_dir 存在后):调用
//!    `bash sync-prototype.sh` 把 a-v{N}.html + bundle.js 复制到 Apps/{code}/

use crate::composer::app_tsx_template::render_app_tsx_template;
use crate::composer::resolve_project_root;
use crate::composer::ComposerError;
use crate::state::progress_event;
use crate::state::{AgentProgress, AppMeta, FlowPlan};
use serde_json::json;
use std::path::{Path, PathBuf};

/// 在 staging 阶段生成 app.tsx 并调用 prototype-tool.js build。
///
/// 产出:
/// - `{stage_dir}/llm-tsx/app.tsx`(composer 自动生成骨架)
/// - `Pre-Proc/{ns}/Prototypes/Apps/{code}/llm-tsx/app.tsx`(复制,build 期望路径)
/// - `Pre-Proc/{ns}/Prototypes/Apps/{code}/a-v{N}.html`(prototype-tool.js 产物)
/// - `Pre-Proc/{ns}/Prototypes/Apps/{code}/a-v{N}.bundle.js`(esbuild IIFE 产物)
///
/// 不调用 sync-prototype.sh(sync 在 build_app 阶段做,因 final_dir 需先存在)。
///
/// 内建 P3 ESM test-driven 回流:prototype-tool.js build 失败时,把截断后的
/// esbuild/standalone 错误回灌进 `render_app_tsx_template` 上下文并重渲 app.tsx,
/// 再重 build,最多 `evaluate::MAX_EVAL_ITERATIONS`(3) 次,收敛即停。全部失败
/// 才返回 `ComposerError::Validation`,与单遍失败语义保持一致。
pub async fn generate_and_build_app_tsx(
    stage_dir: &Path,
    app_code: &str,
    app_name: &str,
    namespace: &str,
    plan: &FlowPlan,
    app_meta: Option<&AppMeta>,
    files_written: &mut usize,
    on_progress: Option<&(impl Fn(AgentProgress) + Send + Sync)>,
) -> Result<(), ComposerError> {
    let project_root = resolve_project_root();

    // 1. 生成目录(llm-tsx/staging + Pre-Proc prototype)
    let llm_tsx_dir = stage_dir.join("llm-tsx");
    tokio::fs::create_dir_all(&llm_tsx_dir).await?;
    let tsx_path = llm_tsx_dir.join("app.tsx");

    let proto_dir = project_root
        .join("Pre-Proc")
        .join(namespace)
        .join("Prototypes")
        .join("Apps")
        .join(app_code);
    tokio::fs::create_dir_all(&proto_dir).await?;
    let proto_tsx_dir = proto_dir.join("llm-tsx");
    tokio::fs::create_dir_all(&proto_tsx_dir).await?;
    let proto_tsx_path = proto_tsx_dir.join("app.tsx");

    // 2. P3 ESM test-driven 回流:render → copy → build,失败重渲,最多 N 次
    let max_retries = crate::evaluate::MAX_EVAL_ITERATIONS;
    let mut last_error_ctx: Option<String> = None;
    let mut built = false;

    for attempt in 0..max_retries {
        let app_tsx = render_app_tsx_template(
            app_code,
            app_name,
            namespace,
            plan,
            app_meta,
            last_error_ctx.as_deref(),
        );
        tokio::fs::write(&tsx_path, &app_tsx).await?;
        tokio::fs::copy(&tsx_path, &proto_tsx_path).await?;
        if attempt == 0 {
            // tsx 骨架算一次产物写入(多次重渲仍为一个文件)
            *files_written += 1;
        }

        match run_prototype_build(&project_root, &proto_tsx_path).await {
            Ok(()) => {
                built = true;
                break;
            }
            Err(e) => {
                let msg = e.to_string();
                last_error_ctx = Some(truncate_for_comment(&msg, 2000));
                if attempt + 1 >= max_retries {
                    return Err(e);
                }
                // 进入下一次回流:携带错误上下文重渲并重 build
            }
        }
    }

    if !built {
        return Err(ComposerError::Validation(
            "ESM 构建回流异常:未成功也未返回错误".to_string(),
        ));
    }

    // 3. 验证 a-v{N}.html 已生成
    let latest_html = find_latest_prototype(&proto_dir, "a-v").await?;
    *files_written += 1;

    if let Some(cb) = on_progress {
        cb(AgentProgress::new(
            "构建应用",
            83,
            format!(
                "已构建 ESM 原型: {} ({}/{})",
                latest_html
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                namespace,
                app_code
            ),
            progress_event::ARTIFACT_WRITTEN,
            Some(json!({
                "path": format!("Pre-Proc/{}/Prototypes/Apps/{}/{}", namespace, app_code, latest_html.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()),
                "kind": "app_prototype_esm"
            })),
        ));
    }

    Ok(())
}

/// 在 build_app 阶段调用 sync-prototype.sh,把最新 a-v{N}.html 同步到 Apps/{code}/。
///
/// 前置:final_dir 已存在(compose_from_flow_plan 的 staging → rename 完成)。
pub async fn sync_prototype(
    app_code: &str,
    namespace: &str,
    on_progress: Option<&(impl Fn(AgentProgress) + Send + Sync)>,
) -> Result<(), ComposerError> {
    let project_root = resolve_project_root();
    let proto_dir = project_root
        .join("Pre-Proc")
        .join(namespace)
        .join("Prototypes")
        .join("Apps")
        .join(app_code);

    let latest_html = find_latest_prototype(&proto_dir, "a-v").await?;
    let sync_path = project_root.join("scripts/sync-prototype.sh");

    let output = tokio::process::Command::new("bash")
        .arg(&sync_path)
        .arg(&latest_html)
        .current_dir(&project_root)
        .output()
        .await
        .map_err(|e| ComposerError::Validation(format!("sync-prototype.sh 启动失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(ComposerError::Validation(format!(
            "sync-prototype.sh 失败:\n  stderr: {}\n  stdout: {}",
            stderr, stdout
        )));
    }

    if let Some(cb) = on_progress {
        cb(AgentProgress::new(
            "构建应用",
            84,
            format!("已同步原型到 Apps/{}/{}/", namespace, app_code),
            progress_event::ARTIFACT_WRITTEN,
            Some(json!({
                "path": format!("Pre-Proc/{}/Apps/{}/prototype.html", namespace, app_code),
                "kind": "app_prototype_synced"
            })),
        ));
    }

    Ok(())
}

/// 在目录中查找最新版本的 a-v{N}.html 或 b-v{N}.html 文件。
///
/// 按 N 的数值大小排序,返回最大的。
///
/// 使用 tokio::fs::read_dir,可在 current_thread 运行时下安全调用。
pub async fn find_latest_prototype(dir: &Path, prefix: &str) -> Result<PathBuf, ComposerError> {
    let mut versions: Vec<(u32, PathBuf)> = Vec::new();

    let mut entries = tokio::fs::read_dir(dir).await.map_err(|e| {
        ComposerError::Validation(format!("读取原型目录失败 {}: {}", dir.display(), e))
    })?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| ComposerError::Validation(format!("读取目录条目失败: {}", e)))?
    {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // 匹配 {prefix}-v{N}.html
        if !file_name.starts_with(prefix) || !file_name.ends_with(".html") {
            continue;
        }

        // 提取版本号: {prefix}{N}.html → {N}
        // prefix 已含 "a-v",strip 后剩 "{N}.html"
        let version_part = file_name
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(".html"));
        if let Some(version_str) = version_part {
            if let Ok(version) = version_str.parse::<u32>() {
                versions.push((version, path));
            }
        }
    }

    versions.sort_by_key(|(v, _)| *v);

    versions.last().map(|(_, p)| p.clone()).ok_or_else(|| {
        ComposerError::Validation(format!(
            "在 {} 中未找到匹配 {}*.html 的原型文件(prototype-tool.js build 可能未产出)",
            dir.display(),
            prefix
        ))
    })
}

/// 调用 `bun scripts/prototype-tool.js build`(子进程)编译 app.tsx。
///
/// 失败返回 `ComposerError::Validation`,含截断后的 stderr/stdout 供回流回灌。
async fn run_prototype_build(
    project_root: &Path,
    proto_tsx_path: &Path,
) -> Result<(), ComposerError> {
    let tool_path = project_root.join("scripts/prototype-tool.js");
    let output = tokio::process::Command::new("bun")
        .arg(&tool_path)
        .arg("build")
        .arg(proto_tsx_path)
        .current_dir(project_root)
        .output()
        .await
        .map_err(|e| {
            ComposerError::Validation(format!(
                "prototype-tool.js build 启动失败(是否安装 bun?): {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(ComposerError::Validation(format!(
            "prototype-tool.js build 失败:\n  stderr: {}\n  stdout: {}",
            stderr, stdout
        )));
    }

    Ok(())
}

/// 把 build 错误信息截断为适合嵌入 app.tsx 注释块的长度(避免巨型注释)。
///
/// 取前 `max_chars` 个字符(按字节截断,但绝不斩断在 UTF-8 中间)。
fn truncate_for_comment(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{}…(已截断)", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_test_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alioth_compose_test_{}_{}",
            test_name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn test_find_latest_prototype_picks_highest_version() {
        let dir = temp_test_dir("highest_version");
        // 创建 a-v1.html, a-v2.html, a-v10.html
        for v in [1, 2, 10] {
            let path = dir.join(format!("a-v{}.html", v));
            fs::write(&path, "<html></html>").unwrap();
        }
        // 创建无关文件
        fs::write(dir.join("b-v1.html"), "<html></html>").unwrap();
        fs::write(dir.join("readme.md"), "readme").unwrap();

        let latest = find_latest_prototype(&dir, "a-v").await.unwrap();
        assert!(latest.file_name().unwrap().to_str().unwrap() == "a-v10.html");
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_find_latest_prototype_empty_dir_errors() {
        let dir = temp_test_dir("empty_dir");
        let result = find_latest_prototype(&dir, "a-v").await;
        assert!(result.is_err(), "空目录应返回错误");
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_find_latest_prototype_no_matching_files_errors() {
        let dir = temp_test_dir("no_match");
        fs::write(dir.join("b-v1.html"), "<html></html>").unwrap();
        let result = find_latest_prototype(&dir, "a-v").await;
        assert!(result.is_err(), "无匹配文件应返回错误");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_truncate_for_comment_short_passthrough() {
        let s = "esbuild: Transform failed";
        assert_eq!(truncate_for_comment(s, 2000), s, "短错误应原样返回");
    }

    #[test]
    fn test_truncate_for_comment_long_is_cut_with_marker() {
        let long = "x".repeat(5000);
        let out = truncate_for_comment(&long, 2000);
        assert!(out.chars().count() <= 2000 + "…(已截断)".chars().count());
        assert!(out.ends_with("…(已截断)"), "截断后应带标记");
    }

    #[test]
    fn test_truncate_for_comment_no_utf8_split() {
        // 多字节字符(中文)不得被斩断在字符中间
        let s: String = "错误".repeat(2000); // 4000 字符,远超 100
        let out = truncate_for_comment(&s, 100);
        assert!(
            out.chars().all(|c| c == '错'
                || c == '误'
                || c == '…'
                || c == '('
                || c == ')'
                || c == '已'
                || c == '截'
                || c == '断'),
            "不得出现乱码/截断的 UTF-8 残片"
        );
    }
}
