//! Skills — AppAgent 专用技能系统
//!
//! 技能是机器可读的 YAML 工作流定义，位于 `skills/` 目录下。
//! 每个技能是一个 `.yaml` 文件，包含名称、描述、Track/Step 定义、
//! 参考的工具列表和输出校验 Schema。
//!
//! ## 格式
//! ```yaml
//! name: service-codegen
//! description: 从 service.json 生成 Rust CRUD 后端代码
//! version: "1.0"
//! tracks:
//!   - name: 模型层
//!     steps:
//!       - id: 1.1
//!         description: 读取 service.json，生成 model.rs
//!         tools: [read_file, write_file]
//!         schema: { type: object, required: [model_path] }
//! skills_path: "skills/"
//! ```
//!
//! AppAgent 在初始化时扫描 `skills/` 目录加载所有技能。
//! 执行时按 Track/Step 调用 LLM，融合技能指令 + 工具定义。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// 技能定义（YAML 结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 技能名（对应文件名，不含 .yaml）
    #[serde(default)]
    pub name: String,
    /// 描述
    pub description: String,
    /// 版本
    #[serde(default)]
    pub version: String,
    /// 工作流 Track
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// 默认引用的工具
    #[serde(default)]
    pub default_tools: Vec<String>,
}

/// 工作流 Track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Track 名（如 "模型层"）
    pub name: String,
    /// 步骤
    pub steps: Vec<Step>,
}

/// 门禁条件：step 完成后必须通过的结构化检查
///
/// 两种形态：
/// - 纯文件检查：仅设 `output_glob`，无 program → 执行者校验文件存在性
/// - 命令门禁：设置 `program` + `args`，白名单校验后执行，可选附加 `output_glob`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGate {
    /// 可执行程序（白名单：target/debug/ontology-mapping, bun, npx, cargo）
    #[serde(default)]
    pub program: String,
    /// 参数（禁止 shell 拼接）
    #[serde(default)]
    pub args: Vec<String>,
    /// 期望退出码（默认 0，若设了 program 则检查）
    #[serde(default)]
    pub expected_exit_code: i32,
    /// 必须存在的产物 glob（纯文件检查时 program="" 即可）
    #[serde(default)]
    pub output_glob: Option<String>,
    /// 超时秒数（默认 120；纯文件检查时忽略）
    #[serde(default = "default_gate_timeout")]
    pub timeout_sec: u64,
}

const fn default_gate_timeout() -> u64 { 120 }

/// 步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// 步骤 ID
    pub id: String,
    /// 指令（LLM system prompt 片段）
    pub instruction: String,
    /// 本步骤可用工具
    #[serde(default)]
    pub tools: Vec<String>,
    /// 门禁条件（step 完成后按序执行）
    #[serde(default)]
    pub gates: Vec<StepGate>,
    /// @deprecated 由 `gates[].output_glob` 替代。保留反序列化兼容。
    #[serde(default)]
    pub outputs: Vec<String>,
    /// 输出校验 Schema（JSON Schema）
    #[serde(default)]
    pub schema: Option<Value>,
}

impl Skill {
    /// 将旧 `outputs` 字段迁移为 `gates[].output_glob`
    pub fn migrate_outputs_to_gates(&mut self) {
        for track in &mut self.tracks {
            for step in &mut track.steps {
                if step.gates.is_empty() && !step.outputs.is_empty() {
                    step.gates = step
                        .outputs
                        .drain(..)
                        .map(|p| StepGate {
                            program: String::new(),
                            args: vec![],
                            expected_exit_code: 0,
                            output_glob: Some(p),
                            timeout_sec: default_gate_timeout(),
                        })
                        .collect();
                }
            }
        }
    }
}

/// 技能执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    pub skill_name: String,
    pub track_index: usize,
    pub step_index: usize,
    pub artifacts: HashMap<String, String>,
}

// ── 技能注册表 ───────────────────────────────────────────────

/// 从 `skills/` 目录加载技能
/// 从 `skills/` 目录加载技能（支持 app 级覆盖 + 全局回退）
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    global_dir: PathBuf,
    app_dir: Option<PathBuf>,
}

impl SkillRegistry {
    /// 仅全局目录（backward compat）
    pub fn new(global_dir: impl Into<PathBuf>) -> Self {
        Self {
            skills: HashMap::new(),
            global_dir: global_dir.into(),
            app_dir: None,
        }
    }

    /// 全局 + app 级覆盖（同名不覆盖）
    pub fn with_app_dir(global_dir: impl Into<PathBuf>, app_dir: impl Into<PathBuf>) -> Self {
        Self {
            skills: HashMap::new(),
            global_dir: global_dir.into(),
            app_dir: Some(app_dir.into()),
        }
    }

    /// 扫描技能目录，加载所有 .yaml（app 级优先，全局回退不覆盖已加载的）
    pub async fn load_all(&mut self) -> Result<usize, String> {
        let dirs: Vec<&PathBuf> = self
            .app_dir
            .iter()
            .chain(std::iter::once(&self.global_dir))
            .collect();
        let mut count = 0usize;

        for dir in &dirs {
            if !dir.exists() {
                continue;
            }
            let mut entries = tokio::fs::read_dir(dir)
                .await
                .map_err(|e| format!("Read skills dir {}: {}", dir.display(), e))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                    continue;
                }

                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| format!("Read {}: {}", path.display(), e))?;

                match yaml_serde::from_str::<Skill>(&content) {
                    Ok(mut skill) => {
                        if skill.name.is_empty() {
                            skill.name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                        }
                        skill.migrate_outputs_to_gates();
                        // app 级优先，不覆盖已存在的（由 app_dir 先加载的 key 全局不覆盖）
                        self.skills.entry(skill.name.clone()).or_insert_with(|| {
                            count += 1;
                            skill
                        });
                    }
                    Err(e) => {
                        common::telemetry::warn!("Parse skill {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(count)
    }
}

impl SkillRegistry {
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let q = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

/// 技能执行器
pub struct SkillExecutor {
    skill: Skill,
    ctx: SkillContext,
}
impl SkillExecutor {
    pub fn new(skill: Skill, _namespace: String) -> Self {
        Self {
            ctx: SkillContext {
                skill_name: skill.name.clone(),
                track_index: 0,
                step_index: 0,
                artifacts: HashMap::new(),
            },
            skill,
        }
    }

    pub fn skill(&self) -> &Skill {
        &self.skill
    }

    pub fn context(&self) -> &SkillContext {
        &self.ctx
    }

    /// 当前是否有待执行的步骤
    pub fn has_next(&self) -> bool {
        if self.ctx.track_index >= self.skill.tracks.len() {
            return false;
        }
        if self.ctx.step_index >= self.skill.tracks[self.ctx.track_index].steps.len() {
            // 检查是否有下一个 Track
            return self.ctx.track_index + 1 < self.skill.tracks.len();
        }
        true
    }

    /// 推进到下一步，返回当前步骤的引用
    pub fn advance(&mut self) -> Option<&Step> {
        // 当前 Track 还有步骤？
        if self.ctx.step_index < self.skill.tracks[self.ctx.track_index].steps.len() {
            let step = &self.skill.tracks[self.ctx.track_index].steps[self.ctx.step_index];
            self.ctx.step_index += 1;
            return Some(step);
        }
        // 下一个 Track？
        if self.ctx.track_index + 1 < self.skill.tracks.len() {
            self.ctx.track_index += 1;
            self.ctx.step_index = 0;
            return self.advance();
        }
        None
    }

    /// 记录产物
    pub fn record_artifact(&mut self, key: String, value: String) {
        self.ctx.artifacts.insert(key, value);
    }

    /// 构建 LLM prompt（技能上下文 + 当前步骤指令 + 工具列表）
    pub fn build_prompt(&self, tool_registry: &crate::tool_registry::ToolRegistry) -> String {
        if self.ctx.track_index >= self.skill.tracks.len() {
            return String::new();
        }
        let track = &self.skill.tracks[self.ctx.track_index];
        if self.ctx.step_index == 0 {
            return String::new();
        }
        let step = &track.steps[self.ctx.step_index - 1];

        // 收集可用工具
        let mut all_tools = self.skill.default_tools.clone();
        all_tools.extend(step.tools.clone());
        let tool_defs: Vec<&crate::tool_registry::ToolDef> = tool_registry
            .tool_defs()
            .iter()
            .filter(|t| all_tools.contains(&t.name))
            .collect();
        let tools_json = serde_json::to_value(&tool_defs).unwrap_or_default();

        format!(
            r#"## 技能：{name}
{desc}

## 当前阶段：{track_name} / 步骤 {step_id}
{step_instruction}

## 可用工具
{tools}

## 已完成产物
{artifacts}
"#,
            name = self.skill.name,
            desc = self.skill.description,
            track_name = track.name,
            step_id = step.id,
            step_instruction = step.instruction,
            tools = serde_json::to_string_pretty(&tools_json).unwrap_or_default(),
            artifacts = if self.ctx.artifacts.is_empty() {
                "(无)".to_string()
            } else {
                self.ctx
                    .artifacts
                    .iter()
                    .map(|(k, v)| format!("  {}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        )
    }

    /// 判断执行是否完成
    pub fn is_complete(&self) -> bool {
        !self.has_next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_roundtrip() {
        let yaml = r#"
name: test-skill
description: A test skill
version: "1.0"
tracks:
  - name: Track 1
    steps:
      - id: "1.1"
        instruction: "Do step 1"
        tools: [read_file]
      - id: "1.2"
        instruction: "Do step 2"
default_tools: [list_skills]
"#;
        let skill: Skill = yaml_serde::from_str(yaml).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.tracks.len(), 1);
        assert_eq!(skill.tracks[0].steps.len(), 2);
    }

    #[test]
    fn test_executor_advance() {
        let yaml = r#"
name: test
description: test
tracks:
  - name: T1
    steps:
      - id: "1.1"
        instruction: "s1"
      - id: "1.2"
        instruction: "s2"
  - name: T2
    steps:
      - id: "2.1"
        instruction: "s3"
"#;
        let skill: Skill = yaml_serde::from_str(yaml).unwrap();
        let mut exec = SkillExecutor::new(skill, "ns".into());
        assert!(exec.has_next());
        assert_eq!(exec.advance().unwrap().id, "1.1");
        assert_eq!(exec.advance().unwrap().id, "1.2");
        assert_eq!(exec.advance().unwrap().id, "2.1");
        assert!(!exec.has_next());
        assert!(exec.advance().is_none());
    }

    #[test]
    fn test_step_gate_roundtrip() {
        let yaml = r#"
name: test-skill
description: A test skill
version: "1.0"
tracks:
  - name: Track 1
    steps:
      - id: "1.1"
        instruction: "Do step 1"
        gates:
          - program: "bun"
            args: ["scripts/prototype-tool.js", "build", "path/to/module.tsx"]
            output_glob: "output/a-v*.html"
            timeout_sec: 60
          - program: "npx"
            args: ["tsc", "--noEmit"]
            expected_exit_code: 0
"#;
        let skill: Skill = yaml_serde::from_str(yaml).unwrap();
        let gates = &skill.tracks[0].steps[0].gates;
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0].program, "bun");
        assert_eq!(gates[0].args, vec!["scripts/prototype-tool.js", "build", "path/to/module.tsx"]);
        assert_eq!(gates[0].output_glob.as_deref(), Some("output/a-v*.html"));
        assert_eq!(gates[0].expected_exit_code, 0);
        assert_eq!(gates[0].timeout_sec, 60);
        assert_eq!(gates[1].program, "npx");
        assert_eq!(gates[1].args, vec!["tsc", "--noEmit"]);
        assert_eq!(gates[1].expected_exit_code, 0);
        assert!(gates[1].output_glob.is_none());
        assert_eq!(gates[1].timeout_sec, 120);

        // 序列化再回读
        let json = yaml_serde::to_string(&skill).unwrap();
        let deser: Skill = yaml_serde::from_str(&json).unwrap();
        let gates2 = &deser.tracks[0].steps[0].gates;
        assert_eq!(gates2.len(), 2);
    }

    #[test]
    fn test_legacy_outputs_compat() {
        let yaml = r#"
name: compat
description: legacy format
tracks:
  - name: T1
    steps:
      - id: "1.1"
        instruction: "s1"
        outputs: ["path/to/output.txt"]
"#;
        // 反序列化兼容性：outputs 字段保持可读
        let skill: Skill = yaml_serde::from_str(yaml).unwrap();
        let step = &skill.tracks[0].steps[0];
        assert_eq!(step.outputs, vec!["path/to/output.txt"]);
        assert!(step.gates.is_empty());

        // 加载时迁移：migrate_outputs_to_gates 后 outputs → gates[].output_glob
        let mut migrated: Skill = yaml_serde::from_str(yaml).unwrap();
        migrated.migrate_outputs_to_gates();
        let m = &migrated.tracks[0].steps[0];
        assert_eq!(m.gates.len(), 1);
        assert_eq!(m.gates[0].output_glob.as_deref(), Some("path/to/output.txt"));
        assert!(m.gates[0].program.is_empty());
        assert!(m.outputs.is_empty());
    }

    #[test]
    fn test_load_real_adapters() {
        // 从 workspace 根加载真实 skill-adapters 目录
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // app-agent crate 在 Meta/backend/app-agent/，skill-adapters 在项目根
        let adapters_dir = manifest_dir.join("../../../skill-adapters");
        if !adapters_dir.exists() {
            eprintln!("Skipping: skill-adapters/ not found at {:?}", adapters_dir);
            return;
        }
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut registry = SkillRegistry::new(&adapters_dir);
        let count = rt.block_on(registry.load_all()).unwrap();
        assert!(count > 0, "应当至少加载 1 个 adapter");
        assert_eq!(count, 6, "应当加载全部 6 个 adapter");
        assert!(registry.get("alioth-module").is_some(), "缺少 alioth-module");
        assert!(registry.get("alioth-block").is_some(), "缺少 alioth-block");
        assert!(registry.get("alioth-ontology").is_some(), "缺少 alioth-ontology");
        assert!(registry.get("alioth-gui").is_some(), "缺少 alioth-gui");
        assert!(registry.get("alioth-service").is_some(), "缺少 alioth-service");
        assert!(registry.get("spec-audit").is_some(), "缺少 spec-audit");

        // 验证每个 adapter 的 steps 都有 instruction 且 gates 可解析
        for name in &["alioth-module", "alioth-block", "alioth-ontology", "alioth-gui", "alioth-service", "spec-audit"] {
            let skill = registry.get(name).unwrap();
            assert!(!skill.tracks.is_empty(), "skill {} 无 tracks", name);
            for (ti, track) in skill.tracks.iter().enumerate() {
                for (si, step) in track.steps.iter().enumerate() {
                    assert!(!step.instruction.is_empty(), "{}.tracks[{}].steps[{}] 缺 instruction", name, ti, si);
                    for (gi, gate) in step.gates.iter().enumerate() {
                        // 命令门禁必须有 program
                        if gate.output_glob.is_none() {
                            assert!(!gate.program.is_empty(), "{}.tracks[{}].steps[{}].gates[{}] 缺 program（output_glob 也空）", name, ti, si, gi);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_gate_default_timeout() {
        let gate: StepGate = yaml_serde::from_str(r#"{
            program: "npx",
            args: ["tsc", "--noEmit"]
        }"#).unwrap();
        assert_eq!(gate.timeout_sec, 120);
        assert_eq!(gate.expected_exit_code, 0);
        assert!(gate.output_glob.is_none());
        assert_eq!(gate.program, "npx");
        assert_eq!(gate.args, vec!["tsc", "--noEmit"]);
}
}

