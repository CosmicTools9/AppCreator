//! 集成测试：从实际 sysint-app 扩展目录加载并验证
//!
//! 验证 ExtensionLoader 能正确解析 Pre-Proc/Apps/sysint-app/extensions/ 下的所有文件。
//! 这是 sysint-app 的"端到端" 校验：YAML 解析 + schema 兼容性 + 业务规则。

use std::path::PathBuf;

fn ext_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = Framework/backend/runtime-engine
    // 父父 = Framework
    // 父父父 = workspace root
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap() // backend
        .parent()
        .unwrap() // Framework
        .parent()
        .unwrap() // workspace root
        .join("Pre-Proc/Apps/sysint-app/extensions")
}

#[test]
fn load_sysint_app_extensions() {
    let dir = ext_dir();
    assert!(dir.exists(), "extensions dir must exist: {}", dir.display());

    let ext = runtime_engine::ExtensionLoader::load_from_dir("sysint-app", &dir)
        .expect("ExtensionLoader::load_from_dir should succeed");

    println!("=== sysint-app extension counts ===");
    println!("  constraints:     {}", ext.constraints.len());
    println!("  business_rules:  {}", ext.business_rules.len());
    println!("  state_machines:  {}", ext.state_machines.len());
    println!("  workflows:       {}", ext.workflows.len());
    println!("  model_profiles:  {}", ext.model_profiles.len());

    // 期望下限（实际数字应 >= 这些）
    assert!(
        ext.constraints.len() >= 25,
        "expected >= 25 constraints, got {}",
        ext.constraints.len()
    );
    assert!(
        ext.business_rules.len() >= 20,
        "expected >= 20 rules, got {}",
        ext.business_rules.len()
    );
    assert!(
        ext.state_machines.len() >= 5,
        "expected >= 5 state machines, got {}",
        ext.state_machines.len()
    );
    assert!(
        ext.workflows.len() >= 5,
        "expected >= 5 workflows, got {}",
        ext.workflows.len()
    );
    assert!(
        ext.model_profiles.len() >= 3,
        "expected >= 3 profiles, got {}",
        ext.model_profiles.len()
    );
}

#[test]
fn state_machine_states_consistent() {
    let dir = ext_dir();
    let ext = runtime_engine::ExtensionLoader::load_from_dir("sysint-app", &dir).unwrap();

    for sm in &ext.state_machines {
        let names: std::collections::HashSet<&str> =
            sm.states.iter().map(|s| s.name.as_str()).collect();

        assert!(
            names.contains(sm.initial_state.as_str()),
            "state machine '{}' (state_field={}) has initial_state '{}' not in states {:?}",
            sm.entity,
            sm.state_field,
            sm.initial_state,
            names
        );

        for t in &sm.transitions {
            assert!(
                names.contains(t.to.as_str()),
                "state machine '{}': transition event='{}' to='{}' not in states {:?}",
                sm.entity,
                t.event,
                t.to,
                names
            );
            for f in &t.from {
                assert!(
                    names.contains(f.as_str()),
                    "state machine '{}': transition event='{}' from='{}' not in states {:?}",
                    sm.entity,
                    t.event,
                    f,
                    names
                );
            }
        }
    }
}

#[test]
fn all_constraints_have_entity() {
    let dir = ext_dir();
    let ext = runtime_engine::ExtensionLoader::load_from_dir("sysint-app", &dir).unwrap();

    for c in &ext.constraints {
        assert!(!c.entity.is_empty(), "constraint entity must be non-empty");
        assert!(
            !c.expression.is_empty(),
            "constraint '{}' expression must be non-empty",
            c.entity
        );
    }

    for r in &ext.business_rules {
        assert!(!r.entity.is_empty(), "rule entity must be non-empty");
        assert!(!r.name.is_empty(), "rule name must be non-empty");
    }
}

#[test]
fn profiles_have_expected_keys() {
    let dir = ext_dir();
    let ext = runtime_engine::ExtensionLoader::load_from_dir("sysint-app", &dir).unwrap();

    // 必须包含全量启用 profile
    assert!(
        ext.model_profiles.contains_key("full_syseng"),
        "missing full_syseng profile"
    );

    // 必须包含至少 4 个领域 profile
    for name in &[
        "space_syseng",
        "equipment_syseng",
        "energy_syseng",
        "infra_syseng",
    ] {
        assert!(
            ext.model_profiles.contains_key(*name),
            "missing domain profile: {}",
            name
        );
    }
}
