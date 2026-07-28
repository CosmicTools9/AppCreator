/**
 * 从 React Hook Form 的 FieldErrors 中递归提取所有错误消息
 *
 * @param errors - RHF FieldErrors 对象
 * @param prefix - 内部递归用的字段路径前缀
 * @returns 错误消息字符串数组
 *
 * @example
 * ```typescript
 * const errors = {
 *   name: { message: "字段名称不能为空" },
 *   referenceConfig: { localKey: { message: "本表外键字段不能为空" } }
 * };
 * extractFieldErrorMessages(errors);
 * // ["字段名称不能为空", "本表外键字段不能为空"]
 * ```
 */
export function extractFieldErrorMessages(
  errors: Record<string, unknown>,
  prefix = "",
): string[] {
  const messages: string[] = [];

  for (const [key, value] of Object.entries(errors)) {
    if (!value) continue;

    if (typeof value === "object" && value !== null) {
      const obj = value as Record<string, unknown>;
      if (
        "message" in obj &&
        typeof obj.message === "string" &&
        obj.message.length > 0
      ) {
        messages.push(obj.message);
      } else {
        // 递归处理嵌套错误（如 referenceConfig.localKey）
        messages.push(...extractFieldErrorMessages(obj, prefix ? `${prefix}.${key}` : key));
      }
    }
  }

  return messages;
}

/**
 * 安全获取任意错误的用户友好消息
 *
 * @param error - 任意错误值
 * @returns 错误消息字符串
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof (error as Record<string, unknown>).message === "string"
  ) {
    return (error as { message: string }).message;
  }
  return "components.error.unknown";
}
