//! AppCreator in-memory data store.
//! Thread-safe HashMap backed. Swappable to sqlx when DB migration is ready.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::RwLock;

use crate::models::*;

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct AppStore {
    pub projects: RwLock<HashMap<i64, Project>>,
    pub templates: RwLock<HashMap<i64, Template>>,
    pub builds: RwLock<HashMap<i64, Build>>,
    pub deployments: RwLock<HashMap<i64, Deployment>>,
}

impl AppStore {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(1, Template {
            id: 1, name: "管理后台".into(), description: "用户管理、权限配置、操作日志".into(),
            category: "management".into(),
        });
        templates.insert(2, Template {
            id: 2, name: "审批流程".into(), description: "报销、请假、合同审核".into(),
            category: "approval".into(),
        });
        templates.insert(3, Template {
            id: 3, name: "ERP 模块".into(), description: "采购、库存、订单".into(),
            category: "erp".into(),
        });
        templates.insert(4, Template {
            id: 4, name: "数据看板".into(), description: "销售报表、运营指标".into(),
            category: "dashboard".into(),
        });

        Self {
            projects: RwLock::new(HashMap::new()),
            templates: RwLock::new(templates),
            builds: RwLock::new(HashMap::new()),
            deployments: RwLock::new(HashMap::new()),
        }
    }

    // ── Projects ────────────────────────────────────

    pub async fn list_projects(&self, _namespace: &str, pagination: &PaginationParams) -> (Vec<Project>, i64) {
        let mut list: Vec<Project> = self.projects.read().await.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = list.len() as i64;
        let offset = pagination.offset() as usize;
        let limit = pagination.per_page() as usize;
        (list.into_iter().skip(offset).take(limit).collect(), total)
    }

    pub async fn get_project(&self, id: i64) -> Option<Project> {
        self.projects.read().await.get(&id).cloned()
    }

    pub async fn create_project(&self, req: CreateProjectRequest, user_id: i64) -> Project {
        let now = Utc::now();
        let project = Project {
            id: next_id(), name: req.name, namespace: req.namespace,
            description: req.description, status: "draft".into(),
            config: req.config, template_id: req.template_id,
            created_by: user_id, created_at: now, updated_at: now,
        };
        self.projects.write().await.insert(project.id, project.clone());
        project
    }

    pub async fn update_project(&self, id: i64, req: UpdateProjectRequest) -> Option<Project> {
        let mut projects = self.projects.write().await;
        let project = projects.get_mut(&id)?;
        if let Some(name) = req.name { project.name = name; }
        if let Some(desc) = req.description { project.description = desc; }
        if let Some(config) = req.config { project.config = config; }
        project.updated_at = Utc::now();
        Some(project.clone())
    }

    pub async fn delete_project(&self, id: i64) -> bool {
        self.projects.write().await.remove(&id).is_some()
    }

    // ── Templates ───────────────────────────────────

    pub async fn list_templates(&self) -> Vec<Template> {
        let mut list: Vec<Template> = self.templates.read().await.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub async fn get_template(&self, id: i64) -> Option<Template> {
        self.templates.read().await.get(&id).cloned()
    }

    // ── Builds ──────────────────────────────────────

    pub async fn create_build(&self, project_id: i64) -> Build {
        let build = Build { id: next_id(), project_id, status: "pending".into(), log: String::new(), created_at: Utc::now() };
        self.builds.write().await.insert(build.id, build.clone());
        build
    }

    pub async fn list_builds(&self, project_id: i64) -> Vec<Build> {
        let mut list: Vec<Build> = self.builds.read().await
            .values().filter(|b| b.project_id == project_id).cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub async fn get_build(&self, id: i64) -> Option<Build> {
        self.builds.read().await.get(&id).cloned()
    }

    // ── Deployments ─────────────────────────────────

    pub async fn create_deployment(&self, project_id: i64, build_id: i64, target: String) -> Deployment {
        let dep = Deployment { id: next_id(), project_id, build_id, status: "pending".into(), target, created_at: Utc::now() };
        self.deployments.write().await.insert(dep.id, dep.clone());
        dep
    }

    pub async fn list_deployments(&self, project_id: i64) -> Vec<Deployment> {
        let mut list: Vec<Deployment> = self.deployments.read().await
            .values().filter(|d| d.project_id == project_id).cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }
}
