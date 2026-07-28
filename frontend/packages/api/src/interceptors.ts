import type {
  RequestConfig,
  RequestInterceptor,
  ResponseInterceptor,
  ErrorInterceptor,
  ApiError,
} from "./types.js";

/**
 * 错误类型枚举
 */
export enum ErrorType {
  /** 网络错误（超时、断网） */
  NETWORK_ERROR = "NETWORK_ERROR",
  /** HTTP 错误（4xx, 5xx） */
  HTTP_ERROR = "HTTP_ERROR",
  /** 验证错误（400, 422） */
  VALIDATION_ERROR = "VALIDATION_ERROR",
  /** 认证错误（401, 403） */
  AUTH_ERROR = "AUTH_ERROR",
  /** 超时错误 */
  TIMEOUT_ERROR = "TIMEOUT_ERROR",
  /** 未知错误 */
  UNKNOWN_ERROR = "UNKNOWN_ERROR",
}

/**
 * 分类后的 API 错误
 */
export interface CategorizedApiError extends ApiError {
  /** 错误类型 */
  type: ErrorType;
  /** HTTP 状态码 */
  statusCode?: number;
  /** 是否需要重试 */
  retryable: boolean;
  /** 原始错误 */
  originalError?: Error;
}

/**
 * 重试配置
 */
export interface RetryConfig {
  /** 最大重试次数 */
  maxRetries: number;
  /** 基础延迟时间（毫秒） */
  baseDelay: number;
  /** 最大延迟时间（毫秒） */
  maxDelay: number;
  /** 退避乘数 */
  backoffMultiplier: number;
  /** 需要重试的状态码 */
  retryableStatusCodes: number[];
}

/**
 * 默认重试配置
 */
export const defaultRetryConfig: RetryConfig = {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 30000,
  backoffMultiplier: 2,
  retryableStatusCodes: [408, 429, 500, 502, 503, 504],
};

/**
 * 错误事件监听器类型
 */
export type ErrorEventListener = (error: CategorizedApiError) => void;

/**
 * 错误事件发射器
 */
class ErrorEventEmitter {
  private listeners: Set<ErrorEventListener> = new Set();

  /**
   * 订阅错误事件
   * @param listener 错误监听器
   * @returns 取消订阅函数
   */
  subscribe(listener: ErrorEventListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * 发射错误事件
   * @param error 分类后的错误
   */
  emit(error: CategorizedApiError): void {
    this.listeners.forEach((listener) => {
      try {
        listener(error);
      } catch (e) {
        console.error("Error in error event listener:", e);
      }
    });
  }

  /**
   * 获取监听器数量
   */
  get listenerCount(): number {
    return this.listeners.size;
  }
}

/**
 * 全局错误事件发射器
 */
export const errorEventEmitter = new ErrorEventEmitter();

/**
 * 错误分类函数
 * @param error 原始 API 错误
 * @param statusCode HTTP 状态码
 * @returns 分类后的错误
 */
export function categorizeError(
  error: ApiError,
  statusCode?: number,
  originalError?: Error,
): CategorizedApiError {
  let type = ErrorType.UNKNOWN_ERROR;
  let retryable = false;

  // 根据状态码分类
  if (statusCode) {
    if (statusCode === 401 || statusCode === 403) {
      type = ErrorType.AUTH_ERROR;
      retryable = false;
    } else if (statusCode === 400 || statusCode === 422) {
      type = ErrorType.VALIDATION_ERROR;
      retryable = false;
    } else if (statusCode >= 500) {
      type = ErrorType.HTTP_ERROR;
      retryable = defaultRetryConfig.retryableStatusCodes.includes(statusCode);
    } else if (statusCode >= 400) {
      type = ErrorType.HTTP_ERROR;
      retryable = false;
    }
  }

  // 根据错误消息分类
  if (error.code === "TIMEOUT" || error.message?.includes("timeout")) {
    type = ErrorType.TIMEOUT_ERROR;
    retryable = true;
  } else if (
    error.code === "NETWORK_ERROR" ||
    error.message?.includes("network") ||
    error.message?.includes("fetch") ||
    originalError?.name === "TypeError"
  ) {
    type = ErrorType.NETWORK_ERROR;
    retryable = true;
  }

  // 为 500 错误提供更具体的错误码和消息
  if (statusCode && statusCode >= 500) {
    const errorCode = error.code || `HTTP_${statusCode}`;
    const fallbackMessage = getServerErrorMessage(statusCode);

    return {
      ...error,
      code: errorCode,
      message: error.message || fallbackMessage,
      type,
      statusCode,
      retryable,
      originalError,
    };
  }

  return {
    ...error,
    type,
    statusCode,
    retryable,
    originalError,
  };
}

/**
 * 获取服务器错误的友好提示消息
 * @param statusCode HTTP 状态码
 * @returns 用户友好的错误消息
 */
function getServerErrorMessage(statusCode: number): string {
  switch (statusCode) {
    case 500:
      return "服务器内部错误，请稍后重试";
    case 502:
      return "网关错误，服务暂时不可用";
    case 503:
      return "服务暂时过载，请稍后重试";
    case 504:
      return "网关超时，服务器响应时间过长";
    default:
      return `服务器错误 (${statusCode})，请稍后重试`;
  }
}

/**
 * 计算重试延迟（指数退避）
 * @param attempt 当前尝试次数
 * @param config 重试配置
 * @returns 延迟时间（毫秒）
 */
export function calculateRetryDelay(
  attempt: number,
  config: RetryConfig = defaultRetryConfig,
): number {
  const delay =
    config.baseDelay * Math.pow(config.backoffMultiplier, attempt - 1);
  return Math.min(delay, config.maxDelay);
}

/**
 * 重试拦截器
 */
export class RetryInterceptor {
  private config: RetryConfig;
  private retryCount: Map<string, number> = new Map();

  constructor(config: Partial<RetryConfig> = {}) {
    this.config = { ...defaultRetryConfig, ...config };
  }

  /**
   * 获取请求的唯一标识
   */
  private getRequestKey(
    config: RequestConfig & { url: string; method: string },
  ): string {
    return `${config.method}:${config.url}`;
  }

  /**
   * 检查是否应该重试
   */
  shouldRetry(
    error: CategorizedApiError,
    config: RequestConfig & { url: string; method: string },
  ): boolean {
    if (!error.retryable) {
      return false;
    }

    const key = this.getRequestKey(config);
    const currentCount = this.retryCount.get(key) || 0;

    return currentCount < this.config.maxRetries;
  }

  /**
   * 获取当前重试次数
   */
  getRetryCount(
    config: RequestConfig & { url: string; method: string },
  ): number {
    return this.retryCount.get(this.getRequestKey(config)) || 0;
  }

  /**
   * 递增重试计数
   */
  incrementRetryCount(
    config: RequestConfig & { url: string; method: string },
  ): void {
    const key = this.getRequestKey(config);
    const currentCount = this.retryCount.get(key) || 0;
    this.retryCount.set(key, currentCount + 1);
  }

  /**
   * 重置重试计数
   */
  resetRetryCount(
    config: RequestConfig & { url: string; method: string },
  ): void {
    this.retryCount.delete(this.getRequestKey(config));
  }

  /**
   * 等待重试延迟
   */
  async waitForRetry(attempt: number): Promise<void> {
    const delay = calculateRetryDelay(attempt, this.config);
    await new Promise((resolve) => setTimeout(resolve, delay));
  }

  /**
   * 清除所有重试计数
   */
  clearAll(): void {
    this.retryCount.clear();
  }
}

/**
 * 日志拦截器
 */
export class LoggingInterceptor {
  private enabled: boolean;

  constructor(enabled = true) {
    this.enabled = enabled;
  }

  /**
   * 记录请求日志
   */
  logRequest(config: RequestConfig & { url: string; method: string }): void {
    if (!this.enabled) return;

    console.log(
      `[API Request] ${config.method} ${config.url}`,
      config.headers ? { headers: config.headers } : "",
    );
  }

  /**
   * 记录响应日志
   */
  logResponse<T>(response: T): void {
    if (!this.enabled) return;

    console.log(`[API Response]`, response);
  }

  /**
   * 记录错误日志
   */
  logError(error: CategorizedApiError): void {
    if (!this.enabled) return;

    console.error(`[API Error] ${error.type}:`, {
      code: error.code,
      message: error.message,
      statusCode: error.statusCode,
      retryable: error.retryable,
      details: error.details,
    });
  }
}

/**
 * 全局日志拦截器实例
 */
export const globalLoggingInterceptor = new LoggingInterceptor(
  import.meta.env.DEV,
);

export class InterceptorManager {
  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];
  private errorInterceptors: ErrorInterceptor[] = [];

  // 请求拦截器
  useRequest(interceptor: RequestInterceptor): () => void {
    this.requestInterceptors.push(interceptor);
    return () => {
      const index = this.requestInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.requestInterceptors.splice(index, 1);
      }
    };
  }

  // 响应拦截器
  useResponse<T>(interceptor: ResponseInterceptor<T>): () => void {
    this.responseInterceptors.push(interceptor as ResponseInterceptor);
    return () => {
      const index = this.responseInterceptors.indexOf(
        interceptor as ResponseInterceptor,
      );
      if (index > -1) {
        this.responseInterceptors.splice(index, 1);
      }
    };
  }

  // 错误拦截器
  useError(interceptor: ErrorInterceptor): () => void {
    this.errorInterceptors.push(interceptor);
    return () => {
      const index = this.errorInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.errorInterceptors.splice(index, 1);
      }
    };
  }

  // 应用请求拦截器
  async applyRequestInterceptors(
    config: RequestConfig & { url: string; method: string },
  ): Promise<RequestConfig & { url: string; method: string }> {
    let result = config;
    for (const interceptor of this.requestInterceptors) {
      result = await interceptor(result);
    }
    return result;
  }

  // 应用响应拦截器
  async applyResponseInterceptors<T>(response: T): Promise<T> {
    let result: unknown = response;
    for (const interceptor of this.responseInterceptors) {
      result = await interceptor(result);
    }
    return result as T;
  }

  // 应用错误拦截器
  async applyErrorInterceptors(error: ApiError): Promise<ApiError> {
    let result = error;
    for (const interceptor of this.errorInterceptors) {
      result = await interceptor(result);
    }
    return result;
  }
}

export const globalInterceptors = new InterceptorManager();
