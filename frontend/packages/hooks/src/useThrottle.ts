import { useState, useEffect, useRef, useCallback } from "react";

/**
 * 节流 Hook - 限制值更新频率
 *
 * @param value 原始值
 * @param interval 节流间隔 (毫秒)，默认 2000ms
 * @returns 节流后的值
 *
 * @example
 * const throttledSearch = useThrottle(searchQuery, 300);
 */
export function useThrottle<T>(value: T, interval: number = 2000): T {
  const [throttledValue, setThrottledValue] = useState<T>(value);
  const lastUpdated = useRef<number>(Date.now());

  useEffect(() => {
    const now = Date.now();
    const timeElapsed = now - lastUpdated.current;

    if (timeElapsed >= interval) {
      // 超过间隔，立即更新
      setThrottledValue(value);
      lastUpdated.current = now;
    } else {
      // 未超过间隔，设置延迟更新
      const timer = setTimeout(() => {
        setThrottledValue(value);
        lastUpdated.current = Date.now();
      }, interval - timeElapsed);

      return () => clearTimeout(timer);
    }
  }, [value, interval]);

  return throttledValue;
}

/**
 * 节流回调函数 Hook - 限制函数执行频率
 *
 * @param callback 原始回调函数
 * @param interval 节流间隔 (毫秒)，默认 2000ms
 * @returns 节流后的回调函数
 *
 * @example
 * const throttledSave = useThrottleCallback(saveToServer, 2000);
 * throttledSave(data); // 最多每 2 秒执行一次
 */
export function useThrottleCallback<T extends (...args: unknown[]) => unknown>(
  callback: T,
  interval: number = 2000,
): (...args: Parameters<T>) => void {
  const lastExecuted = useRef<number>(0);
  const pendingArgs = useRef<Parameters<T> | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const throttledCallback = useCallback(
    (...args: Parameters<T>) => {
      const now = Date.now();
      const timeElapsed = now - lastExecuted.current;

      const execute = () => {
        lastExecuted.current = Date.now();
        pendingArgs.current = null;
        callback(...args);
      };

      if (timeElapsed >= interval) {
        // 超过间隔，立即执行
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
          timeoutRef.current = null;
        }
        execute();
      } else {
        // 未超过间隔，延迟执行
        pendingArgs.current = args;
        if (!timeoutRef.current) {
          timeoutRef.current = setTimeout(() => {
            if (pendingArgs.current) {
              lastExecuted.current = Date.now();
              callback(...pendingArgs.current);
              pendingArgs.current = null;
            }
            timeoutRef.current = null;
          }, interval - timeElapsed);
        }
      }
    },
    [callback, interval],
  );

  return throttledCallback;
}

/**
 * 防抖回调函数 Hook - 延迟执行直到停止触发
 *
 * @param callback 原始回调函数
 * @param delay 延迟时间 (毫秒)，默认 300ms
 * @returns 防抖后的回调函数
 *
 * @example
 * const debouncedSearch = useDebounceCallback(handleSearch, 300);
 */
export function useDebounceCallback<T extends (...args: unknown[]) => unknown>(
  callback: T,
  delay: number = 300,
): (...args: Parameters<T>) => void {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const debouncedCallback = useCallback(
    (...args: Parameters<T>) => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      timeoutRef.current = setTimeout(() => {
        callback(...args);
        timeoutRef.current = null;
      }, delay);
    },
    [callback, delay],
  );

  return debouncedCallback;
}
