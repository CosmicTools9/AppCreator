//! 规约门禁 — App Agent 代码生成后验证
//!
//! 在 CodeGenerator::validate() 中调用，对每个 GeneratedFile
//! 按 file_type 路由到对应的 checker 集合。
//!
//! 覆盖 AGENTS.md 核心边界 + 命名规范 + 数据库规约。

use crate::generator::ir::llm_contract::{GeneratedFile, ValidationError};
use regex::Regex;

/// 规约门禁验证器
pub struct ConventionChecker;

impl ConventionChecker {
    /// 对所有生成文件执行规约检查
    pub fn check_all(files: &[GeneratedFile]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for file in files {
            match file.file_type.as_str() {
                "rust" => Self::check_rust(file, &mut errors),
                "typescript" | "tsx" => Self::check_frontend(file, &mut errors),
                "html" => Self::check_html(file, &mut errors),
                "sql" => {
                    // SQL 文件不应通过 App Agent 生成 —— 所有 schema 在 Rust 中预置
                    errors.push(ValidationError {
                        code: "sql_generation_forbidden".into(),
                        message: "SQL file generation is forbidden. All schema must be preseeded in Rust code (init/seed/migration). See AGENTS.md §执行前预检 Row 1.".into(),
                        file_path: Some(file.file_path.clone()),
                        line_number: None,
                    });
                }
                _ => {}
            }
        }

        errors
    }

    // ── Rust checker ────────────────────────────────────────────────

    fn check_rust(file: &GeneratedFile, errors: &mut Vec<ValidationError>) {
        let content = &file.content;
        let fp = &file.file_path;

        // 1. 禁止直接 DDL
        if let Some(line) = Self::find_line(content, "ALTER TABLE") {
            errors.push(Self::err_at(fp, line, "rust_ddl_forbidden", "Direct 'ALTER TABLE' found. Schema changes must be done via Rust init/seed/migration, not raw SQL."));
        }
        if let Some(line) = Self::find_line(content, "DROP TABLE") {
            errors.push(Self::err_at(fp, line, "rust_ddl_forbidden", "Direct 'DROP TABLE' found. Schema changes must be done via Rust init/seed/migration."));
        }
        if let Some(line) = Self::find_line(content, "CREATE TABLE") {
            errors.push(Self::err_at(fp, line, "rust_ddl_forbidden", "Direct 'CREATE TABLE' found. Schema changes must be done via Rust init/seed/migration."));
        }
        if let Some(line) = Self::find_line(content, "TRUNCATE") {
            errors.push(Self::err_at(
                fp,
                line,
                "rust_ddl_forbidden",
                "Direct 'TRUNCATE' found. Use Rust seed functions instead.",
            ));
        }

        // 2. 禁止 psql 调用
        if let Some(line) = Self::find_line(content, "psql ") {
            errors.push(Self::err_at(fp, line, "psql_forbidden", "Direct 'psql' invocation found. DB interaction must go through sanctioned paths (schema-info, reset-db.sh, or Rust code)."));
        }

        // 3. 禁止 #[sqlx::test] — 必须用 #[tokio::test]
        if let Some(line) = Self::find_line(content, "#[sqlx::test") {
            errors.push(Self::err_at(fp, line, "sqlx_test_forbidden", "'#[sqlx::test]' is forbidden in module backends. Use '#[tokio::test]' + PgPool::connect. See BACKEND_FRAMEWORK.md §5."));
        }

        // 4. 禁止模块内定义 auth 中间件
        if content.contains("AuthMiddleware") || content.contains("auth_middleware") {
            errors.push(Self::err_opt(fp, None, "module_auth_forbidden", "Module-level auth middleware is forbidden. Authentication is handled by Gateway. See BACKEND_FRAMEWORK.md §6.3."));
        }

        // 5. 禁止本地复制 build_cors()
        if let Some(line) = Self::find_line(content, "fn build_cors") {
            errors.push(Self::err_at(fp, line, "local_cors_forbidden", "Local 'build_cors()' is forbidden. Use 'common::build_cors()'. See BACKEND_FRAMEWORK.md §6.2."));
        }

        // 6. 禁止 qk_* 字段定义为非 bigint 类型
        //    Pattern: qk_*: Option<DateTime> / qk_*: Decimal / qk_*: String
        let qk_type_re =
            Regex::new(r"qk_\w+\s*:\s*(Option<)?\s*(DateTime|Decimal|String|bool|f32|f64)")
                .unwrap();
        for (i, line) in content.lines().enumerate() {
            if qk_type_re.is_match(line) {
                errors.push(ValidationError {
                    code: "qk_scalar_type_forbidden".into(),
                    message: format!(
                        "'qk_*' field must be 'Option<i64>' (scalar reference), not DateTime/Decimal/String. See AGENTS.md §标量引用模型. Found: '{}'",
                        line.trim()
                    ),
                    file_path: Some(fp.clone()),
                    line_number: Some((i + 1) as u32),
                });
            }
        }

        // 7. 禁止 use tracing:: — 模块 backend 须用 common::telemetry
        if let Some(line) = Self::find_line(content, "use tracing::") {
            errors.push(Self::err_at(fp, line, "tracing_forbidden", "Direct 'use tracing::' is forbidden in module backends. Use 'common::telemetry'. See AGENTS.md §Never."));
        }

        // 8. 禁止 bigint[] 模拟多对多
        if let Some(line) = Self::find_line(content, "bigint[]") {
            errors.push(Self::err_at(fp, line, "bigint_array_forbidden", "'bigint[]' for many-to-many is forbidden. Use 'zc_id_lifecycle_r_*' relation tables. See AGENTS.md §Never."));
        }
    }

    // ── Frontend (TypeScript/TSX) checker ───────────────────────────

    fn check_frontend(file: &GeneratedFile, errors: &mut Vec<ValidationError>) {
        let content = &file.content;
        let fp = &file.file_path;

        // 1. 禁止 Zustand
        if let Some(line) = Self::find_line(content, "from \"zustand\"") {
            errors.push(Self::err_at(fp, line, "zustand_forbidden", "Zustand import found. Frontend state management must use Jotai v2. See AGENTS.md §Never."));
        }

        // 2. 禁止 Redux
        if let Some(line) = Self::find_line(content, "from \"@reduxjs/toolkit\"") {
            errors.push(Self::err_at(fp, line, "redux_forbidden", "Redux import found. Frontend state management must use Jotai v2. See AGENTS.md §Never."));
        }

        // 3. 禁止 Recoil
        if let Some(line) = Self::find_line(content, "from \"recoil\"") {
            errors.push(Self::err_at(fp, line, "recoil_forbidden", "Recoil import found. Frontend state management must use Jotai v2. See AGENTS.md §Never."));
        }

        // 4. 禁止手动 JOIN（CRUD 必须走 list_refs/get_refs）
        if let Some(line) = Self::find_line(content, ".left_join(") {
            errors.push(Self::err_at(fp, line, "manual_join_forbidden", "Manual '.left_join()' found. CRUD queries must use list_refs/get_refs auto-resolution. See REFERENCE_RESOLVER_SPEC.md."));
        }
    }

    // ── HTML checker ────────────────────────────────────────────────

    fn check_html(file: &GeneratedFile, errors: &mut Vec<ValidationError>) {
        let content = &file.content;
        let fp = &file.file_path;

        // 1. 禁止境外 CDN
        if content.contains("fonts.googleapis.com") || content.contains("fonts.gstatic.com") {
            errors.push(Self::err_opt(
                fp,
                None,
                "foreign_cdn_forbidden",
                "Foreign CDN (Google Fonts) found. See HTML_DESIGN_SPEC §1.2.",
            ));
        }
        if content.contains("use.typekit.net") {
            errors.push(Self::err_opt(
                fp,
                None,
                "foreign_cdn_forbidden",
                "Foreign CDN (Typekit) found. See HTML_DESIGN_SPEC §1.2.",
            ));
        }

        // 2. 原型必须含加载骨架
        if content.contains("text/babel") && !content.contains("#boot-skeleton") {
            errors.push(Self::err_opt(fp, None, "boot_skeleton_missing", "React prototype must contain '#boot-skeleton' loading skeleton. See AGENTS.md §原型性能与稳定性硬规约 §Always Row 1."));
        }

        // 3. React 解构声明
        if content.contains("text/babel") && !content.contains("const { useState, useEffect, useRef, useCallback, Fragment, createElement: h } = React;") {
            errors.push(Self::err_opt(fp, None, "react_deconstruct_missing", "React prototype must contain React destructuring declaration. See AGENTS.md §原型性能与稳定性硬规约 §Always Row 2."));
        }

        // 4. 禁止 \:root CSS 转义
        if content.contains("\\:root") {
            errors.push(Self::err_opt(fp, None, "css_root_escape_forbidden", "CSS '\\:root' (backslash-escaped) found. Must use ':root'. See AGENTS.md §原型性能与稳定性硬规约 §Always Row 4."));
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn find_line(content: &str, pattern: &str) -> Option<u32> {
        for (i, line) in content.lines().enumerate() {
            if line.contains(pattern) {
                return Some((i + 1) as u32);
            }
        }
        None
    }

    /// 快捷构造 ValidationError（含行号）
    fn err_at(file_path: &str, line: u32, code: &str, message: &str) -> ValidationError {
        Self::err_opt(file_path, Some(line), code, message)
    }

    /// 构造 ValidationError（可选行号）
    fn err_opt(file_path: &str, line: Option<u32>, code: &str, message: &str) -> ValidationError {
        ValidationError {
            code: code.to_string(),
            message: message.to_string(),
            file_path: Some(file_path.to_string()),
            line_number: line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Rust checker tests ──────────────────────────────────────────

    #[test]
    fn blocks_alter_table() {
        let file = GeneratedFile {
            file_path: "test.rs".into(),
            content: "sqlx::query!(\"ALTER TABLE x ADD y\");".into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty(), "Should block ALTER TABLE");
        assert!(errs[0].code == "rust_ddl_forbidden");
    }

    #[test]
    fn blocks_psql() {
        let file = GeneratedFile {
            file_path: "script.sh".into(),
            content: "psql -d aliothstudio_dev -c 'SELECT 1'".into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty(), "Should block psql");
    }

    #[test]
    fn blocks_sqlx_test() {
        let file = GeneratedFile {
            file_path: "test.rs".into(),
            content: "#[sqlx::test]\nasync fn t() {}".into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "sqlx_test_forbidden");
    }

    #[test]
    fn blocks_qk_decimal() {
        let file = GeneratedFile {
            file_path: "model.rs".into(),
            content: "pub qk_amount: Option<Decimal>,".into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "qk_scalar_type_forbidden");
    }

    #[test]
    fn allows_qk_i64() {
        let file = GeneratedFile {
            file_path: "model.rs".into(),
            content: "pub qk_amount: Option<i64>,".into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(errs.is_empty(), "qk_*: Option<i64> should be allowed");
    }

    // ── Frontend checker tests ──────────────────────────────────────

    #[test]
    fn blocks_zustand() {
        let file = GeneratedFile {
            file_path: "App.tsx".into(),
            content: "import { create } from \"zustand\"".into(),
            file_type: "tsx".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "zustand_forbidden");
    }

    #[test]
    fn blocks_manual_join() {
        let file = GeneratedFile {
            file_path: "api.rs".into(),
            content: "query.left_join(other)".into(),
            file_type: "typescript".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "manual_join_forbidden");
    }

    // ── HTML checker tests ──────────────────────────────────────────

    #[test]
    fn blocks_google_fonts() {
        let file = GeneratedFile {
            file_path: "index.html".into(),
            content: "<link href=\"https://fonts.googleapis.com/css\">".into(),
            file_type: "html".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "foreign_cdn_forbidden");
    }

    #[test]
    fn blocks_css_root_escape() {
        let file = GeneratedFile {
            file_path: "style.html".into(),
            content: "\\:root { --bg: #fff; }".into(),
            file_type: "html".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
    }

    // ── SQL forbidden test ──────────────────────────────────────────

    #[test]
    fn blocks_sql_generation() {
        let file = GeneratedFile {
            file_path: "schema.sql".into(),
            content: "CREATE TABLE x (id int);".into(),
            file_type: "sql".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(!errs.is_empty());
        assert!(errs[0].code == "sql_generation_forbidden");
    }

    // ── Clean code should pass ──────────────────────────────────────

    #[test]
    fn clean_rust_passes() {
        let file = GeneratedFile {
            file_path: "good.rs".into(),
            content: "use common::build_cors;\nfn init(pool: &PgPool) {\n    // seed data\n}"
                .into(),
            file_type: "rust".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(errs.is_empty(), "Clean code should have no errors");
    }

    #[test]
    fn clean_tsx_passes() {
        let file = GeneratedFile {
            file_path: "good.tsx".into(),
            content: "import { atom } from \"jotai\";\nexport const store = atom(0);".into(),
            file_type: "tsx".into(),
            description: "".into(),
            dependencies: vec![],
        };
        let errs = ConventionChecker::check_all(&[file]);
        assert!(errs.is_empty(), "Jotai-based code should be allowed");
    }
}
