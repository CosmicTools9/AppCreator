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

    fn next_response(&self, prompt: &str) -> Result<String, LlmError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
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

#[async_trait]
impl LlmService for MockLlmService {
    async fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        self.next_response(prompt)
    }

    async fn generate_with_system(&self, _system: &str, prompt: &str) -> Result<String, LlmError> {
        self.next_response(prompt)
    }

    async fn generate_with_params(
        &self,
        _system: &str,
        prompt: &str,
        _overrides: GenerationOverrides,
    ) -> Result<String, LlmError> {
        self.next_response(prompt)
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
