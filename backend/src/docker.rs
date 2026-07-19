//! AppCreator — 版本锁 + 镜像构建编排 + E2E 测试
//!
//! 工作流：
//! 1. `build_images()` → 调用 scripts/gateway/build-docker.sh 构建 Gateway 镜像
//! 2. `generate_lock()` → 记录镜像 tag/digest + Alioth 模型版本
//! 3. `generate_compose()` → 生成引用这些镜像的 E2E compose
//!
//! 不自主实现 Docker 构建（复用 Gateway/SSO build 脚本）。
use crate::{AppConfig, AppCreatorError, BuildOutput};
use std::path::Path;
use std::process::Command;

/// Gateway 镜像构建结果
#[derive(Debug)]
pub struct ImageBuildResult {
    pub backend_tag: String,
    pub frontend_tag: String,
    pub backend_digest: Option<String>,
    pub frontend_digest: Option<String>,
}

/// 调用 scripts/gateway/build-docker.sh 构建 Gateway 镜像
///
/// 通过 --namespace 参数传入目标 namespace，使脚本以 --features {ns} 编译，
/// 确保生成的 Gateway 单体包含该 namespace 下所有服务的路由。
/// 构建后自动记录镜像 digest。
pub fn build_images(
    project_root: &str,
    tag: &str,
    namespace: &str,
) -> Result<ImageBuildResult, AppCreatorError> {
    let script = Path::new(project_root).join("scripts/gateway/build-docker.sh");
    if !script.exists() {
        return Err(AppCreatorError::Build(format!(
            "build script not found: {}",
            script.display()
        )));
    }

    let output = Command::new("bash")
        .arg(&script)
        .arg("--tag")
        .arg(tag)
        .arg("--namespace")
        .arg(namespace)
        .current_dir(project_root)
        .output()
        .map_err(AppCreatorError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppCreatorError::Build(format!(
            "build-docker.sh failed: {}",
            stderr
        )));
    }

    // 记录镜像 digest
    let backend_image = format!("gateway-backend:{}", tag);
    let frontend_image = format!("gateway-frontend:{}", tag);
    let backend_digest = image_digest(&backend_image);
    let frontend_digest = image_digest(&frontend_image);

    Ok(ImageBuildResult {
        backend_tag: backend_image,
        frontend_tag: frontend_image,
        backend_digest,
        frontend_digest,
    })
}

/// 查询本地 Docker 镜像 digest
fn image_digest(image_name: &str) -> Option<String> {
    let output = Command::new("docker")
        .args(["image", "inspect", image_name, "--format", "{{.Id}}"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
/// 版本锁 JSON 模板
const LOCK_JSON: &str = r#"{
  "formatVersion": "1",
  "createdAt": "{{created_at}}",
  "app": {
    "name": "{{name}}",
    "namespace": "{{namespace}}",
    "version": "{{version}}",
    "code": "{{code}}"
  },
  "alioth": {
    "modelVersion": "{{model_version}}"
  },
  "gatewayImages": {
    "backend": "gateway-backend:{{tag}}",
    "frontend": "gateway-frontend:{{tag}}"
  },
  "artifacts": {
    "appJson": "Pre-Proc/{{namespace}}/Apps/{{name}}/app.json",
    "prototypeHtml": "Pre-Proc/{{namespace}}/Apps/{{name}}/prototype.html",
    "extensions": "Pre-Proc/{{namespace}}/Apps/{{name}}/extensions/"
  }
}"#;

/// docker-compose E2E 测试（引用已有 Gateway 镜像）
const COMPOSE_YAML: &str = r#"version: "3.8"
services:
  gateway-backend:
    image: gateway-backend:{{tag}}
    environment:
      ALIOTH_MODEL_VERSION: "{{model_version}}"
    volumes:
      - {{preproc_dir}}:/app/Pre-Proc:ro
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:9001/health"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 15s
    networks:
      - e2e-net

  gateway-frontend:
    image: gateway-frontend:{{tag}}
    ports:
      - "{{frontend_port}}:41717"
    depends_on:
      gateway-backend:
        condition: service_healthy
    networks:
      - e2e-net

  prototype:
    image: nginx:alpine
    volumes:
      - {{proto_file}}:/usr/share/nginx/html/index.html:ro
    ports:
      - "{{proto_port}}:80"
    networks:
      - e2e-net

  e2e:
    image: alpine:3.19
    depends_on:
      gateway-backend:
        condition: service_healthy
      prototype:
        condition: service_started
    command: >
      sh -c "
        echo '=== E2E: {{namespace}}/{{name}} ===' &&
        echo 'Model: {{model_version}}' &&
        wget -qO- http://prototype/ | head -3 &&
        wget -qO- http://gateway-backend:9001/health &&
        echo 'E2E PASSED'
      "
    networks:
      - e2e-net

networks:
  e2e-net:
    driver: bridge
"#;

/// 生成版本锁清单
pub fn generate_lock(config: &AppConfig, gateway_tag: &str) -> String {
    let code = format!(
        "{}-{}",
        config.namespace.to_lowercase(),
        config.name.to_lowercase()
    );
    LOCK_JSON
        .replace("{{created_at}}", &chrono::Utc::now().to_rfc3339())
        .replace("{{name}}", &config.name)
        .replace("{{namespace}}", &config.namespace)
        .replace("{{version}}", &config.version)
        .replace("{{code}}", &code)
        .replace("{{model_version}}", &config.alioth_model_version)
        .replace("{{tag}}", gateway_tag)
}

/// 生成 E2E docker-compose.yml
pub fn generate_compose(config: &AppConfig, gateway_tag: &str, host_ports: &[u16]) -> String {
    let root = Path::new(&config.project_root);
    let preproc_dir = root.join("Pre-Proc").to_string_lossy().to_string();
    let proto_file = root
        .join("Pre-Proc")
        .join(&config.namespace)
        .join("Apps")
        .join(&config.name)
        .join("prototype.html")
        .to_string_lossy()
        .to_string();

    COMPOSE_YAML
        .replace("{{preproc_dir}}", &preproc_dir)
        .replace("{{proto_file}}", &proto_file)
        .replace("{{name}}", &config.name)
        .replace("{{namespace}}", &config.namespace)
        .replace("{{model_version}}", &config.alioth_model_version)
        .replace("{{tag}}", gateway_tag)
        .replace(
            "{{frontend_port}}",
            &host_ports.first().copied().unwrap_or(41717).to_string(),
        )
        .replace(
            "{{proto_port}}",
            &host_ports.get(1).copied().unwrap_or(8081).to_string(),
        )
}

/// 构建
pub fn build(
    config: &AppConfig,
    gateway_tag: &str,
    host_ports: &[u16],
) -> Result<BuildOutput, AppCreatorError> {
    let app_dir = Path::new(&config.project_root)
        .join("Pre-Proc")
        .join(&config.namespace)
        .join("Apps")
        .join(&config.name);

    let proto = app_dir.join("prototype.html");
    if !proto.exists() {
        return Err(AppCreatorError::PrototypeMissing(
            proto.display().to_string(),
        ));
    }

    Ok(BuildOutput {
        lock_content: generate_lock(config, gateway_tag),
        compose_content: generate_compose(config, gateway_tag, host_ports),
        artifacts: vec![
            format!("{}/app-creator.lock", app_dir.display()),
            format!("{}/docker-compose.yml", app_dir.display()),
        ],
    })
}

/// 写入磁盘
pub fn write_artifacts(output: &BuildOutput, app_dir: &Path) -> Result<(), AppCreatorError> {
    std::fs::write(app_dir.join("app-creator.lock"), &output.lock_content)?;
    std::fs::write(app_dir.join("docker-compose.yml"), &output.compose_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_manifest() {
        let cfg = AppConfig {
            name: "inventory".to_string(),
            namespace: "Alioth".to_string(),
            alioth_model_version: "10.3.1".to_string(),
            ..Default::default()
        };
        let lock = generate_lock(&cfg, "v0.2.0");
        assert!(lock.contains("10.3.1"));
        assert!(lock.contains("\"code\": \"alioth-inventory\""));
        assert!(lock.contains("gateway-backend:v0.2.0"));
    }

    #[test]
    fn test_compose_references_gateway_image() {
        let cfg = AppConfig {
            name: "inventory".to_string(),
            namespace: "Alioth".to_string(),
            project_root: "/tmp".to_string(),
            ..Default::default()
        };
        let compose = generate_compose(&cfg, "latest", &[41717, 8081]);
        assert!(compose.contains("image: gateway-backend:latest"));
        assert!(compose.contains("image: gateway-frontend:latest"));
    }
}
