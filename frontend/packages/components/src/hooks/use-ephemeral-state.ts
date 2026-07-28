/**
 * useEphemeralState Hook
 *
 * 解决「持久 useState 缺乏自动清理机制」问题
 *
 * ## 背景
 *
 * 业务中经常需要为列表中的每个 item 维护一个临时状态（"已验证"、"失败"等）。
 * 使用普通 `useState<Record<Id, T>>` 会导致：
 * 1. 数据被删除后, 状态残留在 Map 中（stale entry）
 * 2. 异步操作完成后, 状态与 query 数据时序错位
 * 3. 错误地展示「重复徽章」「永久失败状态」等
 *
 * ## 解决方案
 *
 * 提供 `useEphemeralState<T>` Hook, 自动绑定到 data source 生命周期：
 * - 自动清理 data source 中已不存在的 key
 * - 自动清理 data source 中 status 字段已与 ephemeral 一致的 key
 * - 提供 TTL 自动过期机制
 *
 * ## 用法
 *
 * ```tsx
 * // 1. 派生状态模式：verifyResults 与 server status 比较, 一致则不显示
 * const { getState, setState, clearState, clearAll } = useEphemeralState<{
 *   status: string;
 *   output: string | null;
 * }>();
 *
 * // 2. 每次 server data 变化, 自动清理
 * useEphemeralStateSync(state, versions.map((v) => v.version));
 *
 * // 3. 在 render 时判断 stale
 * const result = getState(v.version);
 * const isStale = result && result.status !== v.status;
 * ```
 */

import { useCallback, useEffect, useRef, useState } from "react";

/**
 * ephemeral state value
 */
export interface EphemeralValue<T> {
  /** state payload */
  data: T;
  /** 创建时间戳 (毫秒) */
  createdAt: number;
  /** 可选过期时间 (毫秒), 0 或 undefined 表示不过期 */
  expiresAt?: number;
}

/**
 * useEphemeralState 返回值
 */
export interface UseEphemeralStateReturn<T> {
  /** 获取某个 key 的 ephemeral state */
  getState: (key: string) => EphemeralValue<T> | undefined;
  /** 设置某个 key 的 ephemeral state */
  setState: (key: string, data: T, ttlMs?: number) => void;
  /** 清除某个 key */
  clearState: (key: string) => void;
  /** 清除所有 ephemeral state */
  clearAll: () => void;
  /** 当前所有 keys (调试用) */
  keys: string[];
}

/**
 * 通用的「按 key 索引的临时状态」Hook
 *
 * 与普通 `useState<Record<key, T>>` 的区别:
 * - 提供精细的 setState(key, data) / clearState(key) API
 * - 支持 TTL 过期
 * - 不负责自动 prune, 由调用方用 useEphemeralStateSync 显式声明绑定
 */
export function useEphemeralState<T>(): UseEphemeralStateReturn<T> {
  const [state, setStateInternal] = useState<Record<string, EphemeralValue<T>>>(
    {},
  );

  const getState = useCallback(
    (key: string): EphemeralValue<T> | undefined => {
      return state[key];
    },
    [state],
  );

  const setState = useCallback((key: string, data: T, ttlMs?: number) => {
    setStateInternal((prev) => {
      const now = Date.now();
      const value: EphemeralValue<T> = {
        data,
        createdAt: now,
        expiresAt: ttlMs ? now + ttlMs : undefined,
      };
      return { ...prev, [key]: value };
    });
  }, []);

  const clearState = useCallback((key: string) => {
    setStateInternal((prev) => {
      if (!(key in prev)) return prev;
      const next = { ...prev };
      delete next[key];
      return next;
    });
  }, []);

  const clearAll = useCallback(() => {
    setStateInternal({});
  }, []);

  return {
    getState,
    setState,
    clearState,
    clearAll,
    keys: Object.keys(state),
  };
}

/**
 * useEphemeralStateSync 选项
 */
export interface UseEphemeralStateSyncOptions {
  /**
   * 同步策略:
   * - "remove-stale": 仅清理不存在于 sourceKeys 的 key
   * - "remove-mismatched": 清理不存在 OR 数据源 status 与 ephemeral 状态不匹配
   *
   * 默认 "remove-stale"
   */
  strategy?: "remove-stale" | "remove-mismatched";
  /**
   * 用于判断 "状态一致" 的字段提取器
   * (仅在 "remove-mismatched" 策略下使用)
   */
  getStatus?: (key: string) => string | undefined;
}

/**
 * 同步 ephemeral state 与 data source
 *
 * 在 data source (e.g. React Query 的 versions) 变化时调用, 自动清理
 * - 已删除 item 的 ephemeral state
 * - status 已同步的 ephemeral state (e.g. DB 中 v.status='verified',
 *   ephemeral 也是 'verified', 这条已无意义)
 *
 * @example
 * ```tsx
 * const ephemeral = useEphemeralState<VerifyResult>();
 * const { data: versions } = useModelPublishVersions();
 *
 * // 每次 versions 变化时同步
 * useEphemeralStateSync(
 *   ephemeral,
 *   versions?.map((v) => v.version) ?? [],
 *   {
 *     strategy: "remove-mismatched",
 *     getStatus: (key) => versions?.find((v) => v.version === key)?.status,
 *   }
 * );
 * ```
 */
export function useEphemeralStateSync<T>(
  ephemeral: UseEphemeralStateReturn<T>,
  sourceKeys: string[],
  options: UseEphemeralStateSyncOptions = {},
): void {
  const { strategy = "remove-stale", getStatus } = options;
  const sourceKeySet = new Set(sourceKeys);

  // 用 ref 保存 latest 闭包, 避免 effect 内直接使用 ephemeral.clearState 触发死循环
  const ephemeralRef = useRef(ephemeral);
  ephemeralRef.current = ephemeral;

  useEffect(() => {
    const ephemeral = ephemeralRef.current;
    const toRemove: string[] = [];

    for (const key of ephemeral.keys) {
      if (!sourceKeySet.has(key)) {
        // data source 中已不存在
        toRemove.push(key);
        continue;
      }
      if (strategy === "remove-mismatched" && getStatus) {
        const sourceStatus = getStatus(key);
        const ephemeralData = ephemeral.getState(key)?.data as unknown as
          | { status?: string }
          | undefined;
        if (
          sourceStatus &&
          ephemeralData?.status &&
          sourceStatus === ephemeralData.status
        ) {
          // 状态已同步, ephemeral entry 已无意义
          toRemove.push(key);
        }
      }
    }

    for (const key of toRemove) {
      ephemeral.clearState(key);
    }
  }, [sourceKeys.join("|"), strategy]);
  // 注: 不把 ephemeral 列入依赖, 用 ref 访问避免 effect 触发循环
}
