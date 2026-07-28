// API 响应类型 —— 从 @alioth/types 重新导出以保持一致性
export type {
  ApiResponse,
  PaginatedData,
  ListQueryParams,
  PaginationParams,
} from "@alioth/types";

// API 特有的类型（请求配置、拦截器等）
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, string[]>;
}

// 请求配置
export interface RequestConfig {
  headers?: Record<string, string>;
  params?: object;
  skipAuth?: boolean;
  timeout?: number;
  /** 跳过缓存 */
  skipCache?: boolean;
  /** 自定义缓存 key */
  cacheKey?: string;
  /** 自定义缓存 TTL（毫秒） */
  cacheTTL?: number;
}

// 拦截器类型
export type RequestInterceptor = (
  config: RequestConfig & { url: string; method: string },
) => RequestConfig & { url: string; method: string };

export type ResponseInterceptor<T = unknown> = (response: T) => T | Promise<T>;

export type ErrorInterceptor = (
  error: ApiError,
) => ApiError | Promise<ApiError>;
