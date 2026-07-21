//! LLM Code Generation Interface Contract
//!
//! LLM 代码生成接口契约 - 定义 LLM 与预制件之间的接口规范
//! 用于指导 LLM 生成符合预制件架构的代码

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代码生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationRequest {
    /// 请求标识
    pub id: String,
    /// 生成目标
    pub target: GenerationTarget,
    /// 本体语义模型
    pub ontology_model: String,
    /// 用户意图描述
    pub user_intent: String,
    /// 预制件接口契约
    pub prefab_contracts: Vec<PrefabContractSpec>,
    /// 上下文信息
    pub context: GenerationContext,
    /// 生成选项
    pub options: GenerationOptions,
}

/// 生成目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationTarget {
    /// 目标类型
    pub target_type: TargetType,
    /// 目标平台
    pub platform: TargetPlatform,
    /// 输出路径
    pub output_path: String,
    /// 模块名称
    pub module_name: String,
}

/// 目标类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    /// 完整应用
    Application,
    /// 业务模块
    Module,
    /// 前端组件
    FrontendComponent,
    /// 后端服务
    BackendService,
    /// API 接口
    ApiInterface,
    /// 数据库迁移
    DatabaseMigration,
    /// 配置文件
    Configuration,
}

/// 目标平台
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetPlatform {
    /// React + TypeScript 前端
    ReactTypeScript,
    /// Rust 后端
    Rust,
    /// PostgreSQL 数据库
    PostgreSQL,
    /// Docker 配置
    Docker,
    /// Kubernetes 配置
    Kubernetes,
    /// 混合平台
    Hybrid(Vec<TargetPlatform>),
}

/// 预制件接口契约规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabContractSpec {
    /// 预制件标识
    pub prefab_id: String,
    /// 预制件类型
    pub prefab_type: PrefabType,
    /// 接口版本
    pub interface_version: String,
    /// 接口定义列表
    pub interfaces: Vec<InterfaceSpec>,
    /// 数据契约
    pub data_contracts: Vec<DataContractSpec>,
    /// 配置模板
    pub configuration_template: Option<String>,
    /// 代码置入规则
    pub placement_rules: Vec<PlacementRule>,
}

/// 预制件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrefabType {
    /// Framework 预制件
    Framework,
    /// Module 预制件
    Module,
    /// Gateway 预制件
    Gateway,
    /// 共享组件
    SharedComponent,
}

/// 接口规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSpec {
    /// 接口名称
    pub name: String,
    /// 接口类型
    pub interface_type: InterfaceType,
    /// 输入参数
    pub inputs: Vec<ParameterSpec>,
    /// 输出参数
    pub outputs: Vec<ParameterSpec>,
    /// 接口描述
    pub description: String,
    /// 使用示例
    pub examples: Vec<String>,
    /// 约束条件
    pub constraints: Vec<String>,
}

/// 接口类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceType {
    /// 数据接口
    Data,
    /// 服务接口
    Service,
    /// 事件接口
    Event,
    /// UI 接口
    Ui,
    /// 配置接口
    Config,
    /// 钩子接口
    Hook,
}

/// 参数规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub parameter_type: String,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 参数描述
    pub description: String,
    /// 验证规则
    pub validation_rules: Vec<String>,
}

/// 数据契约规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataContractSpec {
    /// 契约名称
    pub name: String,
    /// 契约类型
    pub contract_type: DataContractType,
    /// 数据结构定义
    pub schema: String,
    /// 示例数据
    pub examples: Vec<String>,
    /// 约束条件
    pub constraints: Vec<String>,
}

/// 数据契约类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataContractType {
    /// 请求数据
    Request,
    /// 响应数据
    Response,
    /// 事件数据
    Event,
    /// 配置数据
    Config,
    /// 共享数据
    Shared,
}

/// 代码置入规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementRule {
    /// 规则名称
    pub name: String,
    /// 置入类型
    pub placement_type: PlacementType,
    /// 目标文件模式
    pub target_file_pattern: String,
    /// 置入位置
    pub insertion_point: InsertionPoint,
    /// 前置条件
    pub preconditions: Vec<String>,
    /// 后置条件
    pub postconditions: Vec<String>,
}

/// 置入类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlacementType {
    /// 文件创建
    FileCreation,
    /// 代码插入
    CodeInsertion,
    /// 代码替换
    CodeReplacement,
    /// 配置追加
    ConfigAppend,
    /// 依赖注入
    DependencyInjection,
}

/// 插入位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertionPoint {
    /// 位置类型
    pub point_type: InsertionPointType,
    /// 定位标记
    pub marker: String,
    /// 偏移量
    pub offset: i32,
}

/// 插入位置类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InsertionPointType {
    /// 文件开头
    FileStart,
    /// 文件结尾
    FileEnd,
    /// 标记之前
    BeforeMarker,
    /// 标记之后
    AfterMarker,
    /// 替换标记
    ReplaceMarker,
    /// 特定函数内
    InsideFunction,
    /// 特定类内
    InsideClass,
}

/// 生成上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext {
    /// 项目结构
    pub project_structure: ProjectStructure,
    /// 已有代码
    pub existing_code: Vec<ExistingCode>,
    /// 依赖信息
    pub dependencies: Vec<DependencyInfo>,
    /// 环境配置
    pub environment: EnvironmentConfig,
}

/// 项目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// 项目根目录
    pub root_directory: String,
    /// 目录结构
    pub directories: Vec<DirectoryInfo>,
    /// 文件结构
    pub files: Vec<FileInfo>,
}

/// 目录信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    /// 目录路径
    pub path: String,
    /// 目录用途
    pub purpose: String,
    /// 子目录
    pub subdirectories: Vec<String>,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件路径
    pub path: String,
    /// 文件类型
    pub file_type: String,
    /// 文件用途
    pub purpose: String,
    /// 是否由 LLM 生成
    pub is_generated: bool,
}

/// 已有代码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingCode {
    /// 文件路径
    pub file_path: String,
    /// 代码内容
    pub content: String,
    /// 代码用途
    pub purpose: String,
    /// 是否可修改
    pub is_mutable: bool,
}

/// 依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// 依赖名称
    pub name: String,
    /// 依赖版本
    pub version: String,
    /// 依赖类型
    pub dependency_type: DependencyType,
    /// 使用场景
    pub usage: String,
}

/// 依赖类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    /// 生产依赖
    Production,
    /// 开发依赖
    Development,
    /// 对等依赖
    Peer,
    /// 可选依赖
    Optional,
}

/// 环境配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// 目标环境
    pub target_environment: String,
    /// 运行时版本
    pub runtime_version: String,
    /// 数据库配置
    pub database_config: Option<DatabaseConfig>,
    /// 缓存配置
    pub cache_config: Option<CacheConfig>,
    /// 消息队列配置
    pub message_queue_config: Option<MessageQueueConfig>,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型
    pub database_type: String,
    /// 连接字符串
    pub connection_string: String,
    /// 连接池大小
    pub pool_size: u32,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 缓存类型
    pub cache_type: String,
    /// 连接字符串
    pub connection_string: String,
    /// 过期时间
    pub ttl: u64,
}

/// 消息队列配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueueConfig {
    /// 队列类型
    pub queue_type: String,
    /// 连接字符串
    pub connection_string: String,
    /// 主题/队列名称
    pub topics: Vec<String>,
}

/// 生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// 代码风格
    pub code_style: CodeStyle,
    /// 注释级别
    pub comment_level: CommentLevel,
    /// 错误处理策略
    pub error_handling: ErrorHandlingStrategy,
    /// 日志级别
    pub log_level: LogLevel,
    /// 是否包含测试
    pub include_tests: bool,
    /// 是否包含文档
    pub include_docs: bool,
    /// 自定义选项
    pub custom_options: HashMap<String, String>,
}

/// 代码风格
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeStyle {
    /// 标准风格
    Standard,
    /// 紧凑风格
    Compact,
    /// 详细风格
    Verbose,
    /// 自定义风格
    Custom(String),
}

/// 注释级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentLevel {
    /// 无注释
    None,
    /// 最小注释
    Minimal,
    /// 标准注释
    Standard,
    /// 详细注释
    Detailed,
}

/// 错误处理策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
    /// 返回错误
    ReturnError,
    /// 抛出异常
    ThrowException,
    /// 使用 Result 类型
    ResultType,
    /// 使用 Option 类型
    OptionType,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// 错误
    Error,
    /// 警告
    Warn,
    /// 信息
    Info,
    /// 调试
    Debug,
    /// 追踪
    Trace,
}

/// 代码生成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationResponse {
    /// 响应标识
    pub id: String,
    /// 生成状态
    pub status: GenerationStatus,
    /// 生成的文件
    pub generated_files: Vec<GeneratedFile>,
    /// 置入指令
    pub placement_instructions: Vec<PlacementInstruction>,
    /// 元数据
    pub metadata: GenerationMetadata,
    /// 错误信息（如果有）
    pub error: Option<GenerationError>,
}

/// 生成状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStatus {
    /// 成功
    Success,
    /// 部分成功
    PartialSuccess,
    /// 失败
    Failed,
    /// 进行中
    InProgress,
}

/// 生成的文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// 文件路径
    pub file_path: String,
    /// 文件内容
    pub content: String,
    /// 文件类型
    pub file_type: String,
    /// 描述
    pub description: String,
    /// 依赖的文件
    pub dependencies: Vec<String>,
}

/// 置入指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementInstruction {
    /// 指令标识
    pub id: String,
    /// 指令类型
    pub instruction_type: PlacementInstructionType,
    /// 目标文件
    pub target_file: String,
    /// 置入内容
    pub content: String,
    /// 置入位置
    pub insertion_point: InsertionPoint,
    /// 前置条件
    pub preconditions: Vec<String>,
    /// 后置验证
    pub post_validation: Vec<String>,
}

/// 置入指令类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlacementInstructionType {
    /// 创建文件
    CreateFile,
    /// 插入代码
    InsertCode,
    /// 替换代码
    ReplaceCode,
    /// 追加配置
    AppendConfig,
    /// 注册依赖
    RegisterDependency,
    /// 注册路由
    RegisterRoute,
    /// 注册事件
    RegisterEvent,
}

/// 生成元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// 生成时间
    pub generated_at: String,
    /// 使用的模型
    pub model: String,
    /// 令牌使用量
    pub token_usage: TokenUsage,
    /// 生成耗时
    pub generation_time_ms: u64,
    /// 预制件版本
    pub prefab_versions: HashMap<String, String>,
}

/// 令牌使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入令牌数
    pub input_tokens: u64,
    /// 输出令牌数
    pub output_tokens: u64,
    /// 总令牌数
    pub total_tokens: u64,
}

/// 生成错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationError {
    /// 错误代码
    pub code: String,
    /// 错误信息
    pub message: String,
    /// 错误详情
    pub details: Option<String>,
    /// 建议修复
    pub suggestions: Vec<String>,
}

/// 代码生成器接口
pub trait CodeGenerator {
    /// 生成代码
    fn generate(
        &self,
        request: &CodeGenerationRequest,
    ) -> Result<CodeGenerationResponse, GenerationError>;

    /// 验证生成结果
    fn validate(&self, response: &CodeGenerationResponse) -> Result<(), Vec<ValidationError>>;

    /// 获取支持的预制件
    fn supported_prefabs(&self) -> Vec<PrefabContractSpec>;
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误代码
    pub code: String,
    /// 错误信息
    pub message: String,
    /// 相关文件
    pub file_path: Option<String>,
    /// 行号
    pub line_number: Option<u32>,
}

/// LLM 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmServiceConfig {
    /// 服务提供商
    pub provider: LlmProvider,
    /// API 密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// API 基础 URL
    pub base_url: Option<String>,
    /// 超时时间
    pub timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 生成参数
    pub generation_params: GenerationParams,
}

/// LLM 提供商
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    /// DeepSeek
    DeepSeek,
}

/// 生成参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// 温度
    pub temperature: f64,
    /// 最大令牌数
    pub max_tokens: u64,
    /// Top P
    pub top_p: f64,
    /// 频率惩罚
    pub frequency_penalty: f64,
    /// 存在惩罚
    pub presence_penalty: f64,
    /// DeepSeek v4 推理努力度 (low / medium / high)
    /// 控制模型在回答前使用多少链式思考
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 4096,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            reasoning_effort: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation_request() {
        let request = CodeGenerationRequest {
            id: "test-request".to_string(),
            target: GenerationTarget {
                target_type: TargetType::Module,
                platform: TargetPlatform::ReactTypeScript,
                output_path: "/apps/test-module".to_string(),
                module_name: "test-module".to_string(),
            },
            ontology_model: "{\"domains\": []}".to_string(),
            user_intent: "Create an order management module".to_string(),
            prefab_contracts: vec![PrefabContractSpec {
                prefab_id: "framework-crud".to_string(),
                prefab_type: PrefabType::Framework,
                interface_version: "1.0.0".to_string(),
                interfaces: vec![InterfaceSpec {
                    name: "AliothRepository".to_string(),
                    interface_type: InterfaceType::Service,
                    inputs: vec![ParameterSpec {
                        name: "entity".to_string(),
                        parameter_type: "Entity".to_string(),
                        required: true,
                        default_value: None,
                        description: "Entity to operate on".to_string(),
                        validation_rules: vec![],
                    }],
                    outputs: vec![],
                    description: "CRUD operations".to_string(),
                    examples: vec![],
                    constraints: vec![],
                }],
                data_contracts: vec![],
                configuration_template: None,
                placement_rules: vec![PlacementRule {
                    name: "service-registration".to_string(),
                    placement_type: PlacementType::CodeInsertion,
                    target_file_pattern: "src/services/*.rs".to_string(),
                    insertion_point: InsertionPoint {
                        point_type: InsertionPointType::InsideClass,
                        marker: "impl AliothRepository".to_string(),
                        offset: 0,
                    },
                    preconditions: vec![],
                    postconditions: vec![],
                }],
            }],
            context: GenerationContext {
                project_structure: ProjectStructure {
                    root_directory: "/apps".to_string(),
                    directories: vec![],
                    files: vec![],
                },
                existing_code: vec![],
                dependencies: vec![],
                environment: EnvironmentConfig {
                    target_environment: "development".to_string(),
                    runtime_version: "18".to_string(),
                    database_config: None,
                    cache_config: None,
                    message_queue_config: None,
                },
            },
            options: GenerationOptions {
                code_style: CodeStyle::Standard,
                comment_level: CommentLevel::Standard,
                error_handling: ErrorHandlingStrategy::ResultType,
                log_level: LogLevel::Info,
                include_tests: true,
                include_docs: true,
                custom_options: HashMap::new(),
            },
        };

        assert_eq!(request.id, "test-request");
        assert_eq!(request.target.module_name, "test-module");
    }

    #[test]
    fn test_generation_response() {
        let response = CodeGenerationResponse {
            id: "test-response".to_string(),
            status: GenerationStatus::Success,
            generated_files: vec![GeneratedFile {
                file_path: "src/components/OrderList.tsx".to_string(),
                content: "export const OrderList = () => { ... }".to_string(),
                file_type: "typescript".to_string(),
                description: "Order list component".to_string(),
                dependencies: vec![],
            }],
            placement_instructions: vec![PlacementInstruction {
                id: "place-1".to_string(),
                instruction_type: PlacementInstructionType::CreateFile,
                target_file: "src/components/OrderList.tsx".to_string(),
                content: "export const OrderList = () => { ... }".to_string(),
                insertion_point: InsertionPoint {
                    point_type: InsertionPointType::FileStart,
                    marker: "".to_string(),
                    offset: 0,
                },
                preconditions: vec![],
                post_validation: vec![],
            }],
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                model: "deepseek-v4-pro".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 1000,
                    output_tokens: 500,
                    total_tokens: 1500,
                },
                generation_time_ms: 2000,
                prefab_versions: HashMap::new(),
            },
            error: None,
        };

        assert_eq!(response.status, GenerationStatus::Success);
        assert_eq!(response.generated_files.len(), 1);
    }
}
