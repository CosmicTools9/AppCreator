//! App TSX 模板渲染器 — 从 FlowPlan 生成 llm-tsx/app.tsx 骨架。
//!
//! 产出符合 alioth-app SKILL 规约的 AppLayout 组件源文件:
//! - `export default function AppLayout`(named export 不行,必须 default)
//! - 引用 `window.SvgIcon` / `window.ICONS`(由 shell 模板提供)
//! - MODULE_TABS 从 `FlowPlan.used_modules` 推导,用于 TopBar 模块切换
//!
//! 产出后由 `esm_runner::write_app_prototype_esm` 调用
//! `bun scripts/prototype-tool.js build` 编译为 a-v{N}.html。

use crate::state::{AppMeta, FlowPlan};

/// 渲染 app.tsx 模板字符串。
///
/// 参数:
/// - `app_code`: App code(如 "ai-b3ac30776a3a725d")
/// - `app_name`: 展示名(如 "Alioth-Libs")
/// - `namespace`: namespace(如 "Alioth")
/// - `plan`: FlowPlan(提供 used_modules)
/// - `app_meta`: 可选的 LLM 输出元数据(保留参数以保持接口稳定,但 App 级原型不使用 navigation 字段)
/// - `build_error`: 可选的 ESM 回流上下文。由 `esm_runner` 在
///   `prototype-tool.js build` 失败时回灌(截断后的 esbuild/standalone 错误),
///   以 `COMPOSE_BUILD_ERROR` 注释块前缀到产物,供下一次重渲 + 下游 LLM 修复消费。
pub fn render_app_tsx_template(
    app_code: &str,
    app_name: &str,
    namespace: &str,
    plan: &FlowPlan,
    _app_meta: Option<&AppMeta>,
    build_error: Option<&str>,
) -> String {
    let module_tabs_json = render_module_tabs(plan);
    let first_module = plan
        .used_modules
        .first()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let error_banner = match build_error {
        Some(e) => format!("/**\n * COMPOSE_BUILD_ERROR(ESM 回流上下文,重渲时已携带):\n{e}\n */\n"),
        None => String::new(),
    };

    let body = format!(
        r#"// {app_code} app.tsx — {app_name} 应用布局(namespace: {namespace})。
// 由 alioth-compose 技能自动生成骨架,可在此基础上人工精修(见 alioth-app SKILL)。
// App 级原型只负责 AppLayout：TopBar（含 ModuleTabs）+ 对 Module 的嵌入容器。
// Navigation/Footer/滚动视口由 Module embedded 模式自行管理。
// 集成顺序:app(本文件)→ module(<ModuleLayout embedded />)→ block。
import {{ useState }} from 'react';
import ModuleLayout from '../../../Modules/{first_module}/llm-tsx/module';
import {{ createPrototypeLifecycle }} from '../../../_shared/lifecycle';
import {{
  GatewayShell,
  type ModuleTab,
}} from '../../../../../../AppCreator/references/gateway-shell';

const MODULE_TABS: ModuleTab[] = {module_tabs_json};

const DEMO_USER = {{ name: '开发者', email: 'dev@alioth.local', role: 'admin' }};
const WORKSPACE_TRIGGERS = [
  {{ id: 'ai', icon: 'bot', title: 'AI 助手' }},
  {{ id: 'inbox', icon: 'mail', title: '收件箱', unreadCount: 1 }},
  {{ id: 'profile', icon: 'user', title: '个人中心' }},
];

function AppLayout() {{
  const [activeWorkspace, setActiveWorkspace] = useState<string | null>(null);

  return (
    <GatewayShell
      brand="{app_name}"
      brandIcon="gatewayLogo"
      moduleTabs={{MODULE_TABS.map((t) => ({{ ...t, active: t.id === '{first_module}' }}))}}
      searchPlaceholder="搜索模块…"
      user={{DEMO_USER}}
      triggers={{WORKSPACE_TRIGGERS}}
      onTrigger={{(id) => setActiveWorkspace(id)}}
      activeWorkspace={{activeWorkspace}}
      onWorkspaceClose={{() => setActiveWorkspace(null)}}
      hideNavigation
      hideFooter
      noContentScroll
    >
      <div className="flex-1 h-full w-full overflow-hidden">
        <ModuleLayout embedded={{true}} />
      </div>
    </GatewayShell>
  );
}}

window.AppLayout = AppLayout;
export default AppLayout;

export const {{ bootstrap, mount, unmount }} = createPrototypeLifecycle({{
  name: '{app_code}',
  App: AppLayout,
}});
"#,
        app_code = app_code,
        app_name = app_name,
        namespace = namespace,
        module_tabs_json = module_tabs_json,
        first_module = first_module,
    );

    format!("{error_banner}{body}")
}

/// 从 FlowPlan.used_modules 推导 MODULE_TABS JSON。
fn render_module_tabs(plan: &FlowPlan) -> String {
    let tabs: Vec<String> = plan
        .used_modules
        .iter()
        .map(|m| {
            format!(
                "{{ id: '{}', label: '{}', icon: 'box' }}",
                escape_js(m),
                escape_js(m)
            )
        })
        .collect();
    if tabs.is_empty() {
        "[]".to_string()
    } else {
        format!("[\n{}\n]", tabs.join(",\n"))
    }
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::NavGroupMeta;

    fn test_flow_plan(used_modules: Vec<String>, _namespace: &str) -> FlowPlan {
        FlowPlan {
            used_modules,
            namespace: _namespace.to_string(),
            known_entities: vec![],
            workflow_steps: vec![],
            missing_info: vec![],
            created_modules: vec![],
            created_blocks: vec![],
            created_services: vec![],
            ontology_model_json: None,
            functional_units: vec![],
            semantic_concepts: vec![],
            computations: vec![],
            constraints: vec![],
            business_rules: vec![],
            app_meta: None,
        }
    }

    #[test]
    fn test_render_app_tsx_contains_default_export() {
        let plan = test_flow_plan(vec!["system-settings".to_string()], "Alioth");
        let tsx = render_app_tsx_template("ai-test", "TestApp", "Alioth", &plan, None, None);
        assert!(
            tsx.contains("export default AppLayout"),
            "app.tsx 必须含 export default AppLayout"
        );
        assert!(
            tsx.contains("window.AppLayout = AppLayout"),
            "app.tsx 必须挂载到 window.AppLayout"
        );
        assert!(
            tsx.contains("system-settings"),
            "app.tsx 必须含 used_modules"
        );
    }

    #[test]
    fn test_render_app_tsx_contains_module_tabs() {
        let plan = test_flow_plan(
            vec!["inventory".to_string(), "orders".to_string()],
            "Alioth",
        );
        let tsx = render_app_tsx_template("ai-test", "TestApp", "Alioth", &plan, None, None);
        assert!(tsx.contains("MODULE_TABS"), "app.tsx 必须含 MODULE_TABS");
        assert!(
            tsx.contains("GatewayShell"),
            "app.tsx 必须使用 GatewayShell"
        );
        assert!(
            tsx.contains("moduleTabs"),
            "GatewayShell 必须传入 moduleTabs"
        );
        assert!(tsx.contains("inventory"), "moduleTabs 必须含 used_modules");
        assert!(!tsx.contains("NAV_GROUPS"), "app.tsx 不应再含 NAV_GROUPS");
        assert!(!tsx.contains("navGroups"), "App 级不应代管侧边导航");
        assert!(
            tsx.contains("hideNavigation"),
            "App 级 GatewayShell 应隐藏导航"
        );
        assert!(
            tsx.contains("hideFooter"),
            "App 级 GatewayShell 应隐藏 Footer"
        );
        assert!(
            tsx.contains("noContentScroll"),
            "App 级 GatewayShell 应由 Module 自行滚动"
        );
        assert!(
            tsx.contains("ModuleLayout embedded"),
            "App 应以 embedded 模式渲染 ModuleLayout"
        );
    }

    #[test]
    fn test_render_module_tabs_empty() {
        let plan = test_flow_plan(vec![], "Alioth");
        let tsx = render_app_tsx_template("ai-test", "TestApp", "Alioth", &plan, None, None);
        assert!(
            tsx.contains("MODULE_TABS: ModuleTab[] = []"),
            "无模块时 MODULE_TABS 为空数组"
        );
    }

    #[test]
    fn test_render_app_tsx_ignores_navigation_meta() {
        let plan = test_flow_plan(vec!["inventory".to_string()], "Alioth");
        let meta = AppMeta {
            navigation: Some(vec![NavGroupMeta {
                group: "库存管理".to_string(),
                icon: Some("Package".to_string()),
                modules: vec!["inventory".to_string()],
            }]),
            ..Default::default()
        };
        let tsx = render_app_tsx_template("ai-test", "TestApp", "Alioth", &plan, Some(&meta), None);
        assert!(tsx.contains("MODULE_TABS"), "app.tsx 必须含 MODULE_TABS");
        assert!(
            !tsx.contains("库存管理"),
            "app.tsx 不应使用 app_meta.navigation 分组"
        );
    }

    #[test]
    fn test_render_app_tsx_embeds_build_error_banner() {
        let plan = test_flow_plan(vec!["inventory".to_string()], "Alioth");
        let tsx = render_app_tsx_template(
            "ai-test",
            "TestApp",
            "Alioth",
            &plan,
            None,
            Some("esbuild: Transform failed\n  > a-v1.tsx:12:3: Cannot find module"),
        );
        assert!(
            tsx.contains("COMPOSE_BUILD_ERROR"),
            "回流时必须把错误上下文回灌为注释前缀"
        );
        assert!(
            tsx.contains("Cannot find module"),
            "错误正文必须出现在回灌注释中"
        );
        assert!(
            tsx.contains("export default AppLayout"),
            "回灌后骨架主体仍完整"
        );
    }
}
