/**
 * API 缓存系统
 * 提供内存缓存和 localStorage 持久化缓存，支持 TTL 和 LRU 策略
 */

/**
 * 缓存条目接口
 */
export interface CacheEntry<T> {
  /** 缓存数据 */
  data: T;
  /** 缓存时间戳 */
  timestamp: number;
  /** 生存时间（毫秒） */
  ttl: number;
  /** ETag 用于缓存验证 */
  etag?: string;
  /** 访问次数（用于 LRU） */
  accessCount: number;
  /** 最后访问时间 */
  lastAccessed: number;
}

/**
 * 缓存配置选项
 */
export interface CacheConfig {
  /** 默认 TTL（毫秒），默认 5 分钟 */
  defaultTTL: number;
  /** 最大缓存条目数 */
  maxEntries: number;
  /** 是否启用 localStorage */
  enablePersistence: boolean;
  /** localStorage key 前缀 */
  storagePrefix: string;
  /** 是否启用 LRU 淘汰 */
  enableLRU: boolean;
}

/**
 * 缓存统计信息
 */
export interface CacheStats {
  /** 缓存条目数 */
  size: number;
  /** 命中次数 */
  hits: number;
  /** 未命中次数 */
  misses: number;
  /** 命中率 */
  hitRate: number;
  /** 淘汰次数 */
  evictions: number;
}

/**
 * 缓存管理器类
 * 提供内存缓存和持久化缓存功能
 */
export class CacheManager {
  private cache: Map<string, CacheEntry<unknown>> = new Map();
  private config: CacheConfig;
  private stats = {
    hits: 0,
    misses: 0,
    evictions: 0,
  };

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      defaultTTL: 5 * 60 * 1000, // 5 分钟
      maxEntries: 100,
      enablePersistence: true,
      storagePrefix: "api_cache_",
      enableLRU: true,
      ...config,
    };

    // 从 localStorage 恢复缓存
    this.loadFromStorage();
  }

  /**
   * 生成缓存 key
   */
  private generateKey(key: string): string {
    return `${this.config.storagePrefix}${key}`;
  }

  /**
   * 从 localStorage 加载缓存
   */
  private loadFromStorage(): void {
    if (!this.config.enablePersistence || typeof window === "undefined") {
      return;
    }

    try {
      const keys = Object.keys(localStorage).filter((key) =>
        key.startsWith(this.config.storagePrefix),
      );

      for (const storageKey of keys) {
        const raw = localStorage.getItem(storageKey);
        if (raw) {
          const entry: CacheEntry<unknown> = JSON.parse(raw);
          // 只加载未过期的缓存
          if (!this.isEntryExpired(entry)) {
            const key = storageKey.replace(this.config.storagePrefix, "");
            this.cache.set(key, entry);
          } else {
            localStorage.removeItem(storageKey);
          }
        }
      }
    } catch (error) {
      console.warn("[CacheManager] 从 localStorage 加载缓存失败:", error);
    }
  }

  /**
   * 保存缓存到 localStorage
   */
  private saveToStorage(key: string, entry: CacheEntry<unknown>): void {
    if (!this.config.enablePersistence || typeof window === "undefined") {
      return;
    }

    try {
      const storageKey = this.generateKey(key);
      localStorage.setItem(storageKey, JSON.stringify(entry));
    } catch (error) {
      // localStorage 可能已满，清理最旧的条目
      if (error instanceof Error && error.name === "QuotaExceededError") {
        this.evictOldest();
      }
    }
  }

  /**
   * 从 localStorage 删除缓存
   */
  private removeFromStorage(key: string): void {
    if (!this.config.enablePersistence || typeof window === "undefined") {
      return;
    }

    try {
      const storageKey = this.generateKey(key);
      localStorage.removeItem(storageKey);
    } catch (error) {
      console.warn("[CacheManager] 从 localStorage 删除缓存失败:", error);
    }
  }

  /**
   * 检查缓存条目是否过期
   */
  private isEntryExpired(entry: CacheEntry<unknown>): boolean {
    return Date.now() - entry.timestamp > entry.ttl;
  }

  /**
   * 执行 LRU 淘汰
   */
  private evictLRU(): void {
    if (this.cache.size < this.config.maxEntries) {
      return;
    }

    // 找到最久未访问的条目
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    Array.from(this.cache.entries()).forEach(([key, entry]) => {
      if (entry.lastAccessed < oldestTime) {
        oldestTime = entry.lastAccessed;
        oldestKey = key;
      }
    });

    if (oldestKey) {
      this.cache.delete(oldestKey);
      this.removeFromStorage(oldestKey);
      this.stats.evictions++;
    }
  }

  /**
   * 淘汰最旧的条目
   */
  private evictOldest(): void {
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    Array.from(this.cache.entries()).forEach(([key, entry]) => {
      if (entry.timestamp < oldestTime) {
        oldestTime = entry.timestamp;
        oldestKey = key;
      }
    });

    if (oldestKey) {
      this.cache.delete(oldestKey);
      this.removeFromStorage(oldestKey);
      this.stats.evictions++;
    }
  }

  /**
   * 获取缓存
   * @param key 缓存 key
   * @returns 缓存数据，不存在或过期返回 undefined
   */
  get<T>(key: string): T | undefined {
    const entry = this.cache.get(key) as CacheEntry<T> | undefined;

    if (!entry) {
      this.stats.misses++;
      return undefined;
    }

    // 检查是否过期
    if (this.isEntryExpired(entry)) {
      this.cache.delete(key);
      this.removeFromStorage(key);
      this.stats.misses++;
      return undefined;
    }

    // 更新访问统计
    if (this.config.enableLRU) {
      entry.accessCount++;
      entry.lastAccessed = Date.now();
    }

    this.stats.hits++;
    return entry.data;
  }

  /**
   * 设置缓存
   * @param key 缓存 key
   * @param data 缓存数据
   * @param ttl 生存时间（毫秒），默认使用配置值
   * @param etag ETag 用于缓存验证
   */
  set<T>(key: string, data: T, ttl?: number, etag?: string): void {
    // 如果需要，执行 LRU 淘汰
    if (this.config.enableLRU && this.cache.size >= this.config.maxEntries) {
      this.evictLRU();
    }

    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      ttl: ttl ?? this.config.defaultTTL,
      etag,
      accessCount: 1,
      lastAccessed: Date.now(),
    };

    this.cache.set(key, entry as CacheEntry<unknown>);
    this.saveToStorage(key, entry as CacheEntry<unknown>);
  }

  /**
   * 检查缓存是否存在且未过期
   * @param key 缓存 key
   */
  has(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;
    if (this.isEntryExpired(entry)) {
      this.cache.delete(key);
      this.removeFromStorage(key);
      return false;
    }
    return true;
  }

  /**
   * 检查缓存是否过期
   * @param key 缓存 key
   */
  isExpired(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return true;
    return this.isEntryExpired(entry);
  }

  /**
   * 使指定缓存失效
   * @param key 缓存 key
   */
  invalidate(key: string): boolean {
    const existed = this.cache.has(key);
    this.cache.delete(key);
    this.removeFromStorage(key);
    return existed;
  }

  /**
   * 按模式使缓存失效
   * @param pattern 匹配模式（支持通配符 *）
   * @returns 失效的缓存数量
   */
  invalidatePattern(pattern: string): number {
    const regex = new RegExp(pattern.replace(/\*/g, ".*"));
    let count = 0;

    Array.from(this.cache.keys()).forEach((key) => {
      if (regex.test(key)) {
        this.cache.delete(key);
        this.removeFromStorage(key);
        count++;
      }
    });

    return count;
  }

  /**
   * 清空所有缓存
   */
  clear(): void {
    if (this.config.enablePersistence && typeof window !== "undefined") {
      const keys = Object.keys(localStorage).filter((key) =>
        key.startsWith(this.config.storagePrefix),
      );
      for (const key of keys) {
        localStorage.removeItem(key);
      }
    }

    this.cache.clear();
    this.stats.hits = 0;
    this.stats.misses = 0;
    this.stats.evictions = 0;
  }

  /**
   * 获取缓存统计信息
   */
  getStats(): CacheStats {
    const total = this.stats.hits + this.stats.misses;
    return {
      size: this.cache.size,
      hits: this.stats.hits,
      misses: this.stats.misses,
      hitRate: total > 0 ? this.stats.hits / total : 0,
      evictions: this.stats.evictions,
    };
  }

  /**
   * 获取缓存条目的元数据
   * @param key 缓存 key
   */
  getEntryMeta(key: string): Omit<CacheEntry<unknown>, "data"> | undefined {
    const entry = this.cache.get(key);
    if (!entry) return undefined;
    const { data: _, ...meta } = entry;
    return meta;
  }

  /**
   * 获取所有缓存 key
   */
  keys(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * 预热缓存
   * @param key 缓存 key
   * @param fetcher 数据获取函数
   * @param ttl 生存时间
   */
  async prefetch<T>(
    key: string,
    fetcher: () => Promise<T>,
    ttl?: number,
  ): Promise<T> {
    const cached = this.get<T>(key);
    if (cached !== undefined) {
      return cached;
    }

    const data = await fetcher();
    this.set(key, data, ttl);
    return data;
  }

  /**
   * 更新配置
   */
  updateConfig(config: Partial<CacheConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * 获取当前配置
   */
  getConfig(): Readonly<CacheConfig> {
    return { ...this.config };
  }
}

/**
 * 生成缓存 key
 * @param endpoint API 端点
 * @param params 请求参数
 */
export function generateCacheKey(
  endpoint: string,
  params?: Record<string, unknown>,
): string {
  if (!params || Object.keys(params).length === 0) {
    return endpoint;
  }

  const sortedParams = Object.entries(params)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
    .join("&");

  return `${endpoint}?${sortedParams}`;
}

/**
 * 判断是否应该缓存请求
 * @param method HTTP 方法
 * @param endpoint API 端点
 */
export function shouldCache(method: string, endpoint: string): boolean {
  // 只缓存 GET 请求
  if (method.toUpperCase() !== "GET") {
    return false;
  }

  // 不缓存包含敏感信息的端点
  const sensitivePatterns = [
    "/auth",
    "/login",
    "/logout",
    "/token",
    "/password",
  ];

  const lowerEndpoint = endpoint.toLowerCase();
  return !sensitivePatterns.some((pattern) => lowerEndpoint.includes(pattern));
}

/**
 * 全局 API 缓存实例
 */
export const apiCache = new CacheManager({
  defaultTTL: 5 * 60 * 1000, // 5 分钟
  maxEntries: 200,
  enablePersistence: true,
  storagePrefix: "alioth_api_cache_",
  enableLRU: true,
});

/**
 * 短缓存实例（用于频繁变化的数据）
 */
export const shortCache = new CacheManager({
  defaultTTL: 30 * 1000, // 30 秒
  maxEntries: 100,
  enablePersistence: false,
  storagePrefix: "alioth_short_cache_",
  enableLRU: true,
});

/**
 * 长缓存实例（用于不常变化的数据）
 */
export const longCache = new CacheManager({
  defaultTTL: 60 * 60 * 1000, // 1 小时
  maxEntries: 50,
  enablePersistence: true,
  storagePrefix: "alioth_long_cache_",
  enableLRU: true,
});

export default apiCache;
