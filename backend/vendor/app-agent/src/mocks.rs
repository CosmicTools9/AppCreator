//! Mock LLM service for deterministic E2E testing.

use crate::llm::{GenerationOverrides, LlmError, LlmService};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum MockResponse {
    RouterJson(String),
    OntologyJson(String),
    Error(String),
}

/// judge 提示词标志性前缀（见 `evaluate::judge_prompt`），用于 handler mock 识别评审请求。
pub const JUDGE_PROMPT_MARKER: &str = "请评估以下 App 产物在";

pub struct MockLlmService {
    responses: Mutex<Vec<MockResponse>>,
    call_count: AtomicUsize,
    /// 可选 handler：按 prompt 内容动态决定回复（用于区分 judge 评审与其他 LLM 调用）。
    handler: Option<Arc<dyn Fn(&str) -> MockResponse + Send + Sync>>,
}

impl MockLlmService {
    pub fn new(responses: Vec<MockResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            call_count: AtomicUsize::new(0),
            handler: None,
        }
    }

    /// 基于 prompt 内容的 handler 构造 mock（如：judge 返回低分，其余返回合法本体 JSON）。
    pub fn from_handler(handler: impl Fn(&str) -> MockResponse + Send + Sync + 'static) -> Self {
        Self {
            responses: Mutex::new(Vec::new()),
            call_count: AtomicUsize::new(0),
            handler: Some(Arc::new(handler)),
        }
    }

    pub fn router_flash_low() -> Self {
        Self::new(vec![MockResponse::RouterJson(
            r#"{"task_complexity":"moderate","needs_reasoning":false}"#.to_string(),
        )])
    }

    pub fn inventory_planning_ok() -> Self {
        Self::from_handler(|prompt: &str| {
            if prompt.contains(JUDGE_PROMPT_MARKER) {
                MockResponse::OntologyJson("90".to_string())
            } else {
                MockResponse::OntologyJson(inventory_ontology_json())
            }
        })
    }

    pub fn planning_with_clarification() -> Self {
        Self::from_handler(|prompt: &str| {
            if prompt.contains(JUDGE_PROMPT_MARKER) {
                MockResponse::OntologyJson("90".to_string())
            } else {
                MockResponse::OntologyJson(clarification_ontology_json())
            }
        })
    }

    pub fn always_failing(message: &str) -> Self {
        Self::new(vec![MockResponse::Error(message.to_string())])
    }

    /// judge 评审返回高分(90/100 = 0.90),其余 prompt 返回合法仓储管理本体 JSON，
    /// 使 Verifying rubric 评估通过、直接到达 Published 状态。
    pub fn warehouse_planning_ok() -> Self {
        Self::from_handler(|prompt: &str| {
            if prompt.contains(JUDGE_PROMPT_MARKER) {
                MockResponse::OntologyJson("90".to_string())
            } else {
                MockResponse::OntologyJson(warehouse_ontology_json())
            }
        })
    }

    pub fn judge_low_score() -> Self {
        Self::from_handler(|prompt: &str| {
            if prompt.contains(JUDGE_PROMPT_MARKER) {
                MockResponse::OntologyJson("20".to_string())
            } else {
                MockResponse::OntologyJson(inventory_ontology_json())
            }
        })
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    fn next_response(&self, prompt: &str, system: &str) -> Result<String, LlmError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        // ExecutingSkill 步骤：由确定性「技能执行器」产出 tool_calls，
        // 让真实 tool_call 执行（write_file 等）落盘产物，run_command 门禁再真实执行。
        if system.contains("技能执行引擎") {
            if let Some(resp) = skill_actor_response(system) {
                return match resp {
                    MockResponse::RouterJson(s) | MockResponse::OntologyJson(s) => Ok(s),
                    MockResponse::Error(s) => Err(LlmError { message: s }),
                };
            }
        }
        if let Some(h) = &self.handler {
            return match h(prompt) {
                MockResponse::RouterJson(s) | MockResponse::OntologyJson(s) => Ok(s),
                MockResponse::Error(s) => Err(LlmError { message: s }),
            };
        }
        let mut guard = self.responses.lock().expect("mock mutex poisoned");
        if guard.is_empty() {
            return Err(LlmError {
                message: "MockLlmService response queue exhausted".to_string(),
            });
        }
        match guard.remove(0) {
            MockResponse::RouterJson(s) | MockResponse::OntologyJson(s) => Ok(s),
            MockResponse::Error(s) => Err(LlmError { message: s }),
        }
    }
}

/// 解析 system prompt 中的技能名与步骤 ID。
///
/// system 形如：
/// ```text
/// 你是 AppAgent 的技能执行引擎。...
/// 技能：{name}
/// {desc}
/// 阶段：{track_name} / 步骤 {step_id}
/// ```
fn parse_skill_step(system: &str) -> Option<(String, String)> {
    let name = system
        .find("技能：")
        .and_then(|i| {
            let rest = &system[i + "技能：".len()..];
            rest.lines().next().map(|l| l.trim().to_string())
        })?;
    let step = system
        .find("步骤 ")
        .and_then(|i| {
            let rest = &system[i + "步骤 ".len()..];
            // 取直到行尾的 [0-9.]+ 段
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(rest.len());
            let s = rest[..end].trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })?;
    Some((name, step))
}

/// 确定性技能执行器：依据 (技能, 步骤) 返回需写出的产物 (模板路径, 内容)。
///
/// 路径含 `{ns}`/`{module}`/`{block}`/`{app}`/`{service}` 占位符，
/// 由 ExecutingSkill handler 解析为真实路径。仅覆盖「Rust 侧未产出、需 LLM 写」的产物。
fn skill_step_artifacts(skill: &str, step: &str) -> Vec<(String, String)> {
    match (skill, step) {
        ("alioth-module", "1.2") => vec![(
            "Pre-Proc/{ns}/Prototypes/Modules/{module}/capability-map.md".to_string(),
            "# Capability Map\n\n| Capability | Block | Description |\n|---|---|---|\n| list | main | 列表 |\n".to_string(),
        )],
        // 1.4「Block 并行分发」：LLM 产出 block.json（gate 用 Sources/Blocks/*/block.json 通配校验）
        ("alioth-module", "1.4") => vec![(
            "Pre-Proc/{ns}/Sources/Blocks/{module}/block.json".to_string(),
            serde_json::json!({
                "id": "{module}",
                "namespace": "{ns}",
                "block": "SCEN",
                "name": "{module}",
                "version": "0.1.0",
                "prototypeVersion": "b-v1",
                "factors": []
            })
            .to_string(),
        )],
        ("alioth-module", "1.5") => vec![
            (
                "Pre-Proc/{ns}/Prototypes/Modules/{module}/llm-tsx/module.tsx".to_string(),
                "export default function ModulePrototype() { return null; }\n".to_string(),
            ),
            (
                "Pre-Proc/{ns}/Prototypes/Modules/{module}/m-v1.html".to_string(),
                "<!doctype html><html><body>module prototype</body></html>\n".to_string(),
            ),
        ],
        ("alioth-module", "1.6") => vec![(
            "Pre-Proc/{ns}/Prototypes/visual-verify/{module}/v1/report.json".to_string(),
            serde_json::json!({
                "module": "{module}",
                "frames": 7,
                "scores": {"layout": 92, "contrast": 91, "consistency": 93, "readability": 90, "feedback": 92, "accessibility": 91},
                "passed": true
            })
            .to_string(),
        )],
        ("alioth-block", "1.2") | ("alioth-block", "1.3") => vec![
            (
                "Pre-Proc/{ns}/Prototypes/Blocks/{block}/llm-tsx/block.tsx".to_string(),
                "export default function BlockPrototype() { return null; }\n".to_string(),
            ),
            (
                "Pre-Proc/{ns}/Prototypes/Blocks/{block}/b-v1.html".to_string(),
                "<!doctype html><html><body>block prototype</body></html>\n".to_string(),
            ),
        ],
        ("alioth-service", "1.2") => vec![(
            "Pre-Proc/{ns}/Sources/Services/{service}/dto/example.ts".to_string(),
            "export interface ExampleDto { id: string; name: string; }\n".to_string(),
        )],
        ("alioth-service", "1.3") | ("alioth-service", "1.5") => vec![(
            "Pre-Proc/{ns}/Sources/Services/{service}/_verified.json".to_string(),
            serde_json::json!({"verified": true, "step": step}).to_string(),
        )],
        ("alioth-compose", "1.4") => vec![(
            "Pre-Proc/{ns}/Apps/{app}/m-v1.html".to_string(),
            "<!doctype html><html><body>app prototype</body></html>\n".to_string(),
        )],
        ("alioth-compose", "1.5") => vec![(
            "Pre-Proc/{ns}/Apps/{app}/compose-report.json".to_string(),
            serde_json::json!({
                "app": "{app}",
                "modules": [],
                "extensions": ["constraints.yaml", "rules.yaml", "statemachines.yaml", "workflows.yaml"],
                "prototype": "m-v1.html",
                "passed": true
            })
            .to_string(),
        )],
        _ => vec![],
    }
}

/// 构造技能执行器的 LLM 响应（含 write_file tool_calls）。
fn skill_actor_response(system: &str) -> Option<MockResponse> {
    let (skill, step) = parse_skill_step(system)?;
    let artifacts = skill_step_artifacts(&skill, &step);
    let tool_calls: Vec<serde_json::Value> = artifacts
        .into_iter()
        .map(|(path, content)| {
            serde_json::json!({
                "name": "write_file",
                "arguments": {"path": path, "content": content}
            })
        })
        .collect();
    let body = serde_json::json!({
        "completed": true,
        "summary": format!("skill {} step {} executed by mock actor", skill, step),
        "artifacts": {},
        "tool_calls": tool_calls
    });
    Some(MockResponse::OntologyJson(body.to_string()))
}

#[async_trait]
impl LlmService for MockLlmService {
    async fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        self.next_response(prompt, "")
    }

    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<String, LlmError> {
        self.next_response(prompt, system)
    }

    async fn generate_with_params(
        &self,
        system: &str,
        prompt: &str,
        _overrides: GenerationOverrides,
    ) -> Result<String, LlmError> {
        self.next_response(prompt, system)
    }
}

fn inventory_ontology_json() -> String {
    r#"{
  "ontology": {
    "id": "inventory-app",
    "name": "inventory-app",
    "version": "1.0.0",
    "domains": [
      {
        "id": "inventory",
        "name": "inventory",
        "kind": "entity",
        "description": "Inventory records",
        "parent_ids": [],
        "equivalent_ids": [],
        "disjoint_ids": [],
        "properties": [],
        "position": null,
        "prefab_contract": null
      },
      {
        "id": "demand",
        "name": "demand",
        "kind": "entity",
        "description": "Demand forecast",
        "parent_ids": [],
        "equivalent_ids": [],
        "disjoint_ids": [],
        "properties": [],
        "position": null,
        "prefab_contract": null
      }
    ],
    "transaction_lifecycle": null,
    "relations": [],
    "constraints": [],
    "computations": []
  },
  "used_modules": ["inventory", "demand"],
  "known_entities": ["zc_id_inventory", "zc_id_demand"],
  "missing_info": [],
  "workflow_steps": ["List inventory", "View demand"]
}"#
    .to_string()
}

fn clarification_ontology_json() -> String {
    r#"{
  "ontology": {
    "id": "clarify-app",
    "name": "clarify-app",
    "version": "1.0.0",
    "domains": [
      {
        "id": "inventory",
        "name": "inventory",
        "kind": "entity",
        "description": "Inventory records",
        "parent_ids": [],
        "equivalent_ids": [],
        "disjoint_ids": [],
        "properties": [],
        "position": null,
        "prefab_contract": null
      }
    ],
    "transaction_lifecycle": null,
    "relations": [],
    "constraints": [],
    "computations": []
  },
  "used_modules": ["inventory"],
  "known_entities": ["zc_id_inventory"],
  "missing_info": [
    {
      "category": "entity_extension",
      "scene_condition": "User described a custom inventory attribute not present in platform catalog",
      "decision_elements": "Should the agent extend the inventory entity or use an existing property?",
      "judgment_criteria": "Use existing property if semantically equivalent; otherwise propose extension",
      "judgment_result": "Suggest user confirms whether a new custom field is required"
    }
  ],
  "workflow_steps": ["List inventory"]
}"#.to_string()
}

fn warehouse_ontology_json() -> String {
    r#"{
  "ontology": {
    "id": "warehouse-mgmt-app",
    "name": "warehouse-mgmt-app",
    "version": "1.0.0",
    "domains": [
      {
        "id": "warehouse",
        "name": "warehouse",
        "kind": "entity",
        "description": "Warehouse location and storage",
        "parent_ids": [],
        "equivalent_ids": [],
        "disjoint_ids": [],
        "properties": [],
        "position": null,
        "prefab_contract": null
      },
      {
        "id": "inventory",
        "name": "inventory",
        "kind": "entity",
        "description": "Inventory records",
        "parent_ids": [],
        "equivalent_ids": [],
        "disjoint_ids": [],
        "properties": [],
        "position": null,
        "prefab_contract": null
      }
    ],
    "transaction_lifecycle": null,
    "relations": [],
    "constraints": [],
    "computations": []
  },
  "used_modules": ["warehouse-mgmt"],
  "known_entities": ["zc_id_warehouse", "zc_id_inventory"],
  "missing_info": [],
  "workflow_steps": ["View inventory", "Record inbound", "Record outbound"]
}"#
    .to_string()
}
