/**
 * ScalarField · 标量结构化字段 (qk_*)
 *
 * 对齐 DTO_DESIGN_SPEC §2.1–2.2，用于 qk_* 字段。
 * 渲染为单一数值/日期输入，提交结构化对象 `{ value: number | string }`。
 */

import { useT } from "@alioth/i18n";
import { cn } from "../../lib/utils";
import { Input } from "../ui/input";

export interface ScalarFieldProps {
  value: Record<string, unknown> | null | undefined;
  onChange: (value: Record<string, unknown> | null | undefined) => void;
  /** 标量子类型 */
  scalarField?: "price" | "date" | "common";
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function ScalarField({
  value,
  onChange,
  scalarField = "common",
  placeholder,
  disabled,
  className,
}: ScalarFieldProps) {
  const t = useT();
  const currentValue = (
    value && typeof value === "object" ? (value as { value?: unknown }).value : undefined
  ) as string | number | undefined;

  if (scalarField === "date") {
    const dateVal = typeof currentValue === "string" ? currentValue : "";
    return (
      <div className={cn("flex items-center gap-2", className)}>
        <Input
          type="date"
          value={dateVal}
          onChange={(e) => onChange({ value: e.target.value || undefined })}
          disabled={disabled}
          className="flex-1"
        />
      </div>
    );
  }

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Input
        type="number"
        step="any"
        value={currentValue ?? ""}
        onChange={(e) => {
          const num = e.target.value === "" ? undefined : Number(e.target.value);
          onChange({ value: num });
        }}
        placeholder={placeholder ?? t("autoform.scalarValue", {}, { fallback: "数值" })}
        disabled={disabled}
        className="flex-1"
      />
    </div>
  );
}

ScalarField.displayName = "ScalarField";
