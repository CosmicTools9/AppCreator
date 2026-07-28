import { authManager } from "./auth.js";
import { getApiBaseURL } from "./runtime.js";
import {
  globalInterceptors,
  InterceptorManager,
  categorizeError,
  errorEventEmitter,
  RetryInterceptor,
  globalLoggingInterceptor,
  type CategorizedApiError,
  type RetryConfig,
} from "./interceptors.js";
import { apiCache, generateCacheKey, shouldCache } from "./cache.js";
import type { ApiError, RequestConfig } from "./types.js";

export interface ApiClientConfig {
  baseURL?: string;
  timeout?: number;
  headers?: Record<string, string>;
  /** 重试配置 */
  retryConfig?: Partial<RetryConfig>;
  /** 是否启用请求日志 */
  enableLogging?: boolean;
  /** 缓存配置 */
  cache?: {
    /** 是否启用缓存 */
    enabled?: boolean;
    /** 默认 TTL（毫秒） */
    ttl?: number;
    /** 自定义缓存 key 生成器 */
    keyGenerator?: (endpoint: string, params: unknown) => string;
  };
  /**
   * 401 未授权时的回调。返回 true = 刷新成功（跳过登出），false = 刷新失败（继续登出）。
   * 可通过 apiClient.onUnauthorized = ... 在实例化后动态设置。
   */
  onUnauthorized?: () => Promise<boolean>;
}

export class ApiClient {
  private baseURL: string;
  private timeout: number;
  private defaultHeaders: Record<string, string>;
  private interceptorManager: InterceptorManager;
  private retryInterceptor: RetryInterceptor;
  private cacheConfig: NonNullable<ApiClientConfig["cache"]>;
  /** 401 时自动刷新 token 的回调。可在实例化后动态设置。 */
  public onUnauthorized?: () => Promise<boolean>;

  constructor(config: ApiClientConfig = {}) {
    this.baseURL = config.baseURL || getApiBaseURL();
    this.timeout = config.timeout || 30000;
    this.defaultHeaders = {
      "Content-Type": "application/json",
      ...config.headers,
    };
    this.interceptorManager = new InterceptorManager();
    this.retryInterceptor = new RetryInterceptor(config.retryConfig);
    this.cacheConfig = {
      enabled: false,
      ttl: 5 * 60 * 1000, // 5 分钟
      ...config.cache,
    };
    this.onUnauthorized = config.onUnauthorized;

    // 设置默认的JWT拦截器
    this.setupDefaultInterceptors();
  }

  /**
   * 动态更新 baseURL（供容器在加载微前端时调用）
   */
  setBaseURL(url: string): void {
    this.baseURL = url;
  }

  /**
   * 生成缓存 key
   */
  private generateCacheKey(endpoint: string, params?: unknown): string {
    if (this.cacheConfig.keyGenerator) {
      return this.cacheConfig.keyGenerator(endpoint, params);
    }
    return generateCacheKey(
      endpoint,
      params as Record<string, unknown> | undefined,
    );
  }

  /**
   * 获取缓存
   */
  private getCache<T>(key: string): T | undefined {
    if (!this.cacheConfig.enabled) return undefined;
    return apiCache.get<T>(key);
  }

  /**
   * 设置缓存
   */
  private setCache<T>(key: string, data: T, ttl?: number): void {
    if (!this.cacheConfig.enabled) return;
    apiCache.set(key, data, ttl ?? this.cacheConfig.ttl);
  }

  /**
   * 使缓存失效
   */
  private invalidateCache(pattern: string): void {
    apiCache.invalidatePattern(pattern);
  }

  private setupDefaultInterceptors(): void {
    // 请求拦截器：不再添加JWT header; token lives in httpOnly cookie
    this.interceptorManager.useRequest((config) => {
      return config;
    });

    // 错误拦截器：处理401，优先尝试 onUnauthorized 静默刷新
    this.interceptorManager.useError(async (error) => {
      if (error.message?.includes("401") || error.code === "UNAUTHORIZED") {
        if (this.onUnauthorized) {
          const refreshed = await this.onUnauthorized();
          if (refreshed) {
            // 刷新成功，跳过登出 — 调用方重试
            throw error;
          }
        }
        authManager.logout();
        if (!import.meta.env.DEV && !window.location.pathname.startsWith("/auth")) {
          window.location.href = "/auth/login";
        }
      }
      throw error;
    });
  }

  private async request<T>(
    method: string,
    endpoint: string,
    data?: unknown,
    config: RequestConfig = {},
    retryAttempt = 0,
  ): Promise<T> {
    // 生成缓存 key
    const cacheKey = config.cacheKey || this.generateCacheKey(endpoint, config.params as Record<string, unknown> | undefined);
    const shouldUseCache =
      this.cacheConfig.enabled &&
      !config.skipCache &&
      shouldCache(method, endpoint);

    // GET 请求先检查缓存
    if (shouldUseCache && method.toUpperCase() === "GET") {
      const cached = this.getCache<T>(cacheKey);
      if (cached !== undefined) {
        globalLoggingInterceptor.logResponse({
          cached: true,
          endpoint,
          data: cached,
        });
        return cached;
      }
    }

    // 处理查询参数
    let url = `${this.baseURL}${endpoint}`;
    if (config.params && typeof config.params === 'object') {
      const searchParams = new URLSearchParams();
      Object.entries(config.params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      const queryString = searchParams.toString();
      if (queryString) {
        url += (url.includes('?') ? '&' : '?') + queryString;
      }
    }

    // 应用拦截器
    let requestConfig = await this.interceptorManager.applyRequestInterceptors({
      url,
      method,
      ...config,
      headers: {
        ...this.defaultHeaders,
        ...config.headers,
      },
    });

    // 应用全局拦截器
    requestConfig =
      await globalInterceptors.applyRequestInterceptors(requestConfig);

    // 记录请求日志
    globalLoggingInterceptor.logRequest(requestConfig);

    // 创建AbortController用于超时（支持请求级覆盖）
    const controller = new AbortController();
    const timeoutMs = config.timeout ?? this.timeout;
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(requestConfig.url, {
        method: requestConfig.method,
        headers: requestConfig.headers,
        credentials: "include",
        body: data ? JSON.stringify(data) : undefined,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // 解析响应（BigInt-safe：超过 MAX_SAFE_INTEGER 的数值转为字符串）
      const responseText = await response.text().catch(() => "null");
      let result: any = null;
      try {
        result = JSON.parse(responseText, (_key, value) => {
          if (typeof value === "number" && !Number.isSafeInteger(value)) {
            return String(value);
          }
          return value;
        });
      } catch {
        result = null;
      }

      // 应用响应拦截器
      if (result) {
        result = await this.interceptorManager.applyResponseInterceptors(result);
        result = await globalInterceptors.applyResponseInterceptors(result);
        globalLoggingInterceptor.logResponse(result);
      }

      if (!response.ok) {
        const error: ApiError = {
          code: result?.code || `HTTP_${response.status}`,
          message:
            result?.message || `Request failed with status ${response.status}`,
          details: result?.details,
        };

        // 分类错误
        const categorizedError = categorizeError(error, response.status);
        globalLoggingInterceptor.logError(categorizedError);

        // 发射错误事件
        errorEventEmitter.emit(categorizedError);

        // 401 + onUnauthorized 配置：静默刷新 + 重试一次（retryAttempt=10 防循环）
        if (response.status === 401 && this.onUnauthorized && retryAttempt < 10) {
          const refreshed = await this.onUnauthorized();
          if (refreshed) {
            return this.request<T>(method, endpoint, data, config, 10);
          }
        }

        // 检查是否需要重试
        if (
          this.retryInterceptor.shouldRetry(categorizedError, requestConfig)
        ) {
          this.retryInterceptor.incrementRetryCount(requestConfig);
          const retryCount = this.retryInterceptor.getRetryCount(requestConfig);
          await this.retryInterceptor.waitForRetry(retryCount);
          return this.request<T>(method, endpoint, data, config, retryCount);
        }

        throw await this.interceptorManager.applyErrorInterceptors(
          categorizedError,
        );
      }

      // 成功后重置重试计数
      this.retryInterceptor.resetRetryCount(requestConfig);

      // POST/PUT/PATCH/DELETE 请求使相关缓存失效（必须在 204 提前返回前执行）
      if (["POST", "PUT", "PATCH", "DELETE"].includes(method.toUpperCase())) {
        // 使相同端点的 GET 缓存失效
        const baseEndpoint = endpoint.split("?")[0];
        apiCache.invalidatePattern(`${baseEndpoint}*`);

        // REST 资源变更时（如 PUT/DELETE /collections/{id}），父集合列表 /collections 也应失效
        const methodUpper = method.toUpperCase();
        if (methodUpper !== "POST") {
          const segments = baseEndpoint.split("/").filter(Boolean);
          if (segments.length >= 2) {
            const parentPath = "/" + segments.slice(0, -1).join("/");
            apiCache.invalidatePattern(`${parentPath}*`);
          }
        }
      }

      // 处理204 No Content
      if (response.status === 204) {
        return undefined as T;
      }

      // 缓存 GET 请求结果
      if (shouldUseCache && method.toUpperCase() === "GET") {
        this.setCache(cacheKey, result, config.cacheTTL);
      }

      return result as T;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error) {
        if (error.name === "AbortError") {
          const timeoutError = categorizeError(
            { code: "TIMEOUT", message: "Request timeout" },
            undefined,
            error,
          );
          globalLoggingInterceptor.logError(timeoutError);
          errorEventEmitter.emit(timeoutError);
          throw await this.interceptorManager.applyErrorInterceptors(
            timeoutError,
          );
        }

        // 分类网络错误
        const networkError = categorizeError(
          { code: "NETWORK_ERROR", message: error.message },
          undefined,
          error,
        );
        globalLoggingInterceptor.logError(networkError);
        errorEventEmitter.emit(networkError);

        // 检查是否需要重试网络错误
        if (this.retryInterceptor.shouldRetry(networkError, requestConfig)) {
          this.retryInterceptor.incrementRetryCount(requestConfig);
          const retryCount = this.retryInterceptor.getRetryCount(requestConfig);
          await this.retryInterceptor.waitForRetry(retryCount);
          return this.request<T>(method, endpoint, data, config, retryCount);
        }

        throw await this.interceptorManager.applyErrorInterceptors(
          networkError,
        );
      }
      throw error;
    }
  }

  // HTTP方法
  get<T>(endpoint: string, config?: RequestConfig): Promise<T> {
    return this.request<T>("GET", endpoint, undefined, config);
  }

  post<T>(
    endpoint: string,
    data?: unknown,
    config?: RequestConfig,
  ): Promise<T> {
    return this.request<T>("POST", endpoint, data, config);
  }

  put<T>(endpoint: string, data?: unknown, config?: RequestConfig): Promise<T> {
    return this.request<T>("PUT", endpoint, data, config);
  }

  patch<T>(
    endpoint: string,
    data?: unknown,
    config?: RequestConfig,
  ): Promise<T> {
    return this.request<T>("PATCH", endpoint, data, config);
  }

  delete<T>(endpoint: string, data?: unknown, config?: RequestConfig): Promise<T> {
    return this.request<T>("DELETE", endpoint, data, config);
  }

  // 拦截器访问
  get interceptors() {
    return this.interceptorManager;
  }

  /**
   * 手动使缓存失效
   * @param pattern 匹配模式（支持通配符 *）
   */
  invalidateCachePattern(pattern: string): void {
    apiCache.invalidatePattern(pattern);
  }

  /**
   * 清空所有缓存
   */
  clearCache(): void {
    apiCache.clear();
  }

  /**
   * 获取缓存统计信息
   */
  getCacheStats() {
    return apiCache.getStats();
  }

  /**
   * 预取数据
   * @param endpoint API 端点
   * @param fetcher 数据获取函数
   * @param ttl 缓存时间
   */
  async prefetch<T>(
    endpoint: string,
    fetcher: () => Promise<T>,
    ttl?: number,
  ): Promise<T> {
    const cacheKey = this.generateCacheKey(endpoint, undefined);
    return apiCache.prefetch(cacheKey, fetcher, ttl);
  }
}

// 默认客户端实例（保持向后兼容：默认启用缓存）
export const apiClient = new ApiClient({ cache: { enabled: true } });

// 创建自定义配置的客户端
export function createApiClient(config: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}
