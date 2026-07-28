import { useRef, useMemo } from "react";
import {
  useController,
  type Control,
  type FieldValues,
  type Path,
} from "react-hook-form";

/**
 * 浅比较两个对象
 */
export function shallowEqual<T>(a: T, b: T): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object") return false;
  if (a === null || b === null) return false;

  const keysA = Object.keys(a);
  const keysB = Object.keys(b);

  if (keysA.length !== keysB.length) return false;

  for (const key of keysA) {
    if (
      (a as Record<string, unknown>)[key] !==
      (b as Record<string, unknown>)[key]
    ) {
      return false;
    }
  }

  return true;
}

/**
 * 深比较两个值
 */
export function deepEqual<T>(a: T, b: T): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object") return false;
  if (a === null || b === null) return false;

  const keysA = Object.keys(a);
  const keysB = Object.keys(b);

  if (keysA.length !== keysB.length) return false;

  for (const key of keysA) {
    if (!keysB.includes(key)) return false;
    if (
      !deepEqual(
        (a as Record<string, unknown>)[key],
        (b as Record<string, unknown>)[key],
      )
    ) {
      return false;
    }
  }

  return true;
}

/**
 * 表单字段记忆化 Hook 选项
 */
interface UseFormFieldMemoOptions {
  /** 自定义比较函数 */
  compare?: (a: unknown, b: unknown) => boolean;
  /** 调试模式 */
  debug?: boolean;
}

/**
 * 表单字段记忆化 Hook
 *
 * 提供额外的字段级记忆化优化层
 */
export function useFormFieldMemo<
  T extends FieldValues,
  K extends Path<T> = Path<T>,
>(name: K, control: Control<T>, options: UseFormFieldMemoOptions = {}) {
  const { compare = Object.is, debug = false } = options;
  const previousValue = useRef<unknown>(undefined);
  const renderCount = useRef(0);

  const { field, fieldState } = useController({
    name,
    control,
  });

  const isEqual = useMemo(() => {
    const equal = compare(previousValue.current, field.value);

    if (debug) {
      renderCount.current += 1;
      if (!equal) {
        console.log(`[useFormFieldMemo] ${name} changed:`, {
          from: previousValue.current,
          to: field.value,
          renderCount: renderCount.current,
        });
      }
    }

    previousValue.current = field.value;
    return equal;
  }, [field.value, name, compare, debug]);

  return {
    field,
    fieldState,
    isEqual,
    renderCount: renderCount.current,
  };
}

export default useFormFieldMemo;
