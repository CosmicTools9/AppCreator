import { useCallback } from "react";
import type { FieldValues, FieldErrors } from "react-hook-form";
import { useNotification } from "./use-notification";

/**
 * useFormErrorReporter 返回值
 */
interface UseFormErrorReporterReturn {
  /**
   * 将字段验证错误通过 toast 报告给用户
   *
   * @param errors - React Hook Form 的 FieldErrors
   */
  reportFieldErrors: (errors: FieldErrors<FieldValues>) => void;
  /**
   * 生成可直接传给 handleSubmit 的第二个参数（验证失败回调），
   * 自动将字段错误通过 toast 汇报。
   *
   * 用法：
   * ```tsx
   * const { onValidationError } = useFormErrorReporter();
   * <form onSubmit={handleSubmit(onValid, onValidationError)}>
   * ```
   */
  onValidationError: <T extends FieldValues>(
    errors: FieldErrors<T>,
  ) => void;
}

/**
 * useFormErrorReporter Hook
 *
 * 统一表单验证错误汇报机制，将 React Hook Form 的字段级验证错误
 * 自动通过 sonner toast 汇总提示给用户。
 *
 * 与 useNotification 联动，支持嵌套字段（如 referenceConfig.localKey）
 * 的错误消息递归提取。
 *
 * @example
 * ```tsx
 * const { form: { handleSubmit } } = useFieldForm({ ... });
 * const { onValidationError } = useFormErrorReporter();
 *
 * <form onSubmit={handleSubmit(onSubmit, onValidationError)}>
 *   ...
 * </form>
 * ```
 *
 * @example
 * ```tsx
 * // 手动汇报字段错误
 * const { reportFieldErrors } = useFormErrorReporter();
 * reportFieldErrors(form.formState.errors);
 * ```
 */
export function useFormErrorReporter(): UseFormErrorReporterReturn {
  const { notify } = useNotification();

  const reportFieldErrors = useCallback(
    (errors: FieldErrors<FieldValues>) => {
      notify.fieldErrors(errors as Record<string, unknown>);
    },
    [notify],
  );

  const onValidationError = useCallback(
    <T extends FieldValues>(errors: FieldErrors<T>) => {
      notify.fieldErrors(errors as Record<string, unknown>);
    },
    [notify],
  );

  return {
    reportFieldErrors,
    onValidationError,
  };
}

export type { UseFormErrorReporterReturn };
