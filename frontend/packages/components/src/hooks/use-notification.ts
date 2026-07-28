import { toast } from "sonner";
import {
  extractFieldErrorMessages,
  getErrorMessage,
} from "@alioth/utils";
import { useT } from "@alioth/i18n";

/**
 * 通知选项
 */
interface NotificationOptions {
  description?: string;
}

/**
 * 错误通知选项
 */
interface ErrorNotificationOptions extends NotificationOptions {
  action?: () => void;
  actionLabel?: string;
}

/**
 * Promise 通知消息
 */
interface PromiseMessages {
  loading: string;
  success: string;
  error: string;
}

/**
 * useNotification 返回值
 */
interface UseNotificationReturn {
  notify: {
    /**
     * 显示成功通知
     */
    success: (message: string, options?: NotificationOptions) => void;
    /**
     * 显示错误通知
     */
    error: (message: string, options?: ErrorNotificationOptions) => void;
    /**
     * 显示警告通知
     */
    warning: (message: string, options?: NotificationOptions) => void;
    /**
     * 显示信息通知
     */
    info: (message: string, options?: NotificationOptions) => void;
    /**
     * 显示加载通知，返回 toast id
     */
    loading: (message: string) => string | number;
    /**
     * 关闭通知
     */
    dismiss: (toastId?: string | number) => void;
    /**
     * 包装 Promise，自动处理加载/成功/错误状态
     * 返回原始 Promise，以便可以 await
     */
    promise: <T>(promise: Promise<T>, messages: PromiseMessages) => Promise<T>;
    /**
     * 将表单字段验证错误通过 toast 汇报给用户
     * 自动递归提取嵌套字段（如 referenceConfig.localKey）的错误消息
     */
    fieldErrors: (
      errors: Record<string, unknown>,
      options?: NotificationOptions,
    ) => void;
    /**
     * 将任意 API/运行时错误通过 toast 汇报给用户
     * 自动提取最友好的错误消息
     */
    apiError: (
      error: unknown,
      options?: ErrorNotificationOptions,
    ) => void;
  };
}

/**
 * useNotification Hook
 *
 * 基于 sonner 的 Toast 封装，提供统一的通知 API。
 * 涵盖成功、错误、警告、信息、Promise 状态以及表单/API 错误汇报。
 *
 * @example
 * ```tsx
 * const { notify } = useNotification();
 *
 * // 成功通知
 * notify.success("操作成功");
 *
 * // 错误通知带重试
 * notify.error("操作失败", {
 *   action: retryFn,
 *   actionLabel: t("components.action.retry")
 * });
 *
 * // 异步操作
 * await notify.promise(
 *   fetchData(),
 *   { loading: "加载中...", success: "加载完成", error: "加载失败" }
 * );
 *
 * // 表单验证错误
 * notify.fieldErrors(formErrors);
 *
 * // API 错误
 * notify.apiError(apiError);
 * ```
 */
function useNotification(): UseNotificationReturn {
  const t = useT();
  return {
    notify: {
      success: (message: string, options?: NotificationOptions) => {
        toast.success(message, {
          description: options?.description,
        });
      },

      error: (message: string, options?: ErrorNotificationOptions) => {
        toast.error(message, {
          description: options?.description,
          action: options?.action
            ? {
                label: options.actionLabel || t("components.action.retry"),
                onClick: options.action,
              }
            : undefined,
        });
      },

      warning: (message: string, options?: NotificationOptions) => {
        toast.warning(message, {
          description: options?.description,
        });
      },

      info: (message: string, options?: NotificationOptions) => {
        toast.info(message, {
          description: options?.description,
        });
      },

      loading: (message: string): string | number => {
        return toast.loading(message);
      },

      dismiss: (toastId?: string | number) => {
        toast.dismiss(toastId);
      },

      promise: <T>(
        promise: Promise<T>,
        messages: PromiseMessages,
      ): Promise<T> => {
        // 显示加载中通知
        const loadingId = toast.loading(messages.loading);

        // 包装原始 Promise，在完成后关闭加载通知
        return promise
          .then((result) => {
            toast.dismiss(loadingId);
            toast.success(messages.success);
            return result;
          })
          .catch((error) => {
            toast.dismiss(loadingId);
            toast.error(messages.error);
            throw error;
          });
      },

      fieldErrors: (
        errors: Record<string, unknown>,
        options?: NotificationOptions,
      ) => {
        const messages = extractFieldErrorMessages(errors);
        if (messages.length > 0) {
          toast.error(messages.join("；"), {
            description: options?.description,
          });
        }
      },

      apiError: (
        error: unknown,
        options?: ErrorNotificationOptions,
      ) => {
        const message = getErrorMessage(error);
        toast.error(message.startsWith("components.") ? t(message as 'components.error.unknown') : message, {
          description: options?.description,
          action: options?.action
            ? {
                label: options.actionLabel || t("components.action.retry"),
                onClick: options.action,
              }
            : undefined,
        });
      },
    },
  };
}

export { useNotification };
export type {
  UseNotificationReturn,
  NotificationOptions,
  ErrorNotificationOptions,
  PromiseMessages,
};
