/**
 * @description Shared error types for frontend applications
 *
 * 与后端 `alioth_common::ErrorResponse` 对齐的前端错误类型。
 * ErrorType / CategorizedApiError / RetryConfig 等运行时类型由 @alioth/api 提供。
 */

/**
 * @description Unified application error interface for frontend-backend communication
 *
 * 与后端 `alioth_common::ErrorResponse` 对齐：{code, message, details?}
 */
export interface AppError {
  /** Error code for programmatic handling (e.g., "UNAUTHORIZED", "NOT_FOUND") */
  code: string;
  /** Human-readable error message */
  message: string;
  /** Additional error details (field errors, validation issues, etc.) */
  details?: Record<string, unknown>;
  /** Unix timestamp in milliseconds when error occurred */
  timestamp?: number;
}

/**
 * @description Type guard to check if a value is an AppError
 */
export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as AppError).code === "string" &&
    typeof (value as AppError).message === "string"
  );
}
