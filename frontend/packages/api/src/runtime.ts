/**
 * API 运行时配置
 *
 * 支持三级注入（优先级从高到低）：
 * 1. 运行时注入: window.__ALIOTH_API_BASE_URL__
 * 2. 构建时环境变量: import.meta.env.VITE_API_BASE_URL
 * 3. 降级默认值: '/api'
 */
export function getApiBaseURL(): string {
  return "/api";
}

/**
 * 设置运行时 API baseURL（供容器在加载微前端时调用）
 */
export function setApiBaseURL(url: string): void {
  if (typeof window !== "undefined") {
    (window as any).__ALIOTH_API_BASE_URL__ = url;
  }
}
