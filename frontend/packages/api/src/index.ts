// 运行时配置
export { getApiBaseURL, setApiBaseURL } from "./runtime.js";

// 核心客户端
export { ApiClient, apiClient, createApiClient } from "./client.js";
export type { ApiClientConfig } from "./client.js";

// 微前端生命周期工厂
export { createMicroAppLifecycle } from "./micro-app.js";
export type { MicroAppProps, MicroAppLifecycle, CreateMicroAppOptions } from "./micro-app.js";

// 模块组件注册表
export { ModuleComponentRegistry, moduleRegistry, useModuleComponent } from "./registry.js";
export type { RegisteredComponent } from "./registry.js";

// 模块独立入口工厂
export { bootstrapModule } from "./bootstrap.js";
export type { BootstrapOptions } from "./bootstrap.js";

// AI 聊天服务
export { createAIChatService } from "./ai-chat.js";
export type { AIChatService, AIChatMessage, AIChatConfig, AIChatContext, AgentInfo, ChatSessionStore } from "./ai-chat.js";
export { InMemorySessionStore } from "./ai-chat.js";

// 认证管理
export { tokenManager, authManager } from "./auth.js";
export type { TokenPair } from "./auth.js";

// 拦截器
export {
  InterceptorManager,
  globalInterceptors,
  errorEventEmitter,
  categorizeError,
  RetryInterceptor,
  LoggingInterceptor,
  globalLoggingInterceptor,
  ErrorType,
  defaultRetryConfig,
} from "./interceptors.js";
export type {
  CategorizedApiError,
  RetryConfig,
  ErrorEventListener,
} from "./interceptors.js";

// 缓存系统
export {
  CacheManager,
  apiCache,
  shortCache,
  longCache,
  generateCacheKey,
  shouldCache,
} from "./cache.js";
export type { CacheEntry, CacheConfig, CacheStats } from "./cache.js";



// 类型
export type {
  ApiResponse,
  PaginatedData,
  ListQueryParams,
  PaginationParams,
  ApiError,
  RequestConfig,
  RequestInterceptor,
  ResponseInterceptor,
  ErrorInterceptor,
} from "./types.js";

// OpenAPI Generation
export {
  generateClientFromOpenAPI,
  generateReactQueryHooks,
  type OpenAPISpec,
  type OpenAPIOperation,
  type OpenAPIParameter,
  type OpenAPIRequestBody,
  type OpenAPIResponse,
  type OpenAPISchema,
  type ReactQueryHooksConfig,
} from "./openapi.js";

// React Query Hooks Factory
export {
  createQueryHook,
  createApiQueryHook,
  createMutationHook,
  createApiMutationHook,
  createCrudHooks,
  createPaginatedCrudHooks,
  type QueryHookConfig,
  type MutationHookConfig,
  type ApiQueryOptions,
  type ApiMutationOptions,
  type CrudHooksConfig,
  type CrudHooks,
  type PaginatedCrudHooksConfig,
  type PaginatedCrudHooks,
} from "./query-hooks.js";

// 审批系统 API
export { createApprovalApi } from "./approval.js";
export type { ApprovalApi, ApprovalApiConfig } from "./approval.js";
