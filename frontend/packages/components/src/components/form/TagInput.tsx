/**
 * TagInput · 标签输入
 *
 * 通过 Enter / 逗号 / 空格 / blur 添加标签。
 * 已添加标签以可移除的 Badge 展示。
 * 支持粘贴以逗号/空格分隔的多个标签。
 *
 * Value: string[]
 */

import * as React from "react";
import { X } from "lucide-react";

import { cn } from "../../lib/utils";
import { Badge } from "../ui/badge";
import { Input } from "../ui/input";
import { useT } from "@alioth/i18n";

export interface TagInputProps {
  value: string[];
  onChange: (value: string[]) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  /** 输入时触发（用于异步校验、查重等） */
  onInputChange?: (text: string) => void;
  /** 限制最大标签数量 */
  max?: number;
  /** 单个标签最大字符数（超出自动截断） */
  maxLength?: number;
  /** 是否允许重复，默认去重 */
  allowDuplicates?: boolean;
  /** 添加分隔符正则，默认匹配逗号、空白 */
  separators?: RegExp;
}

export function TagInput({
  value,
  onChange,
  placeholder,
  disabled,
  className,
  onInputChange,
  max,
  maxLength = 32,
  allowDuplicates = false,
  separators = /[\s,;]+/,
}: TagInputProps) {
  const t = useT();
  const effectivePlaceholder = placeholder ?? t("form.tags.placeholder", {}, { fallback: "输入后按回车添加" });

  const [draft, setDraft] = React.useState("");
  const inputRef = React.useRef<HTMLInputElement>(null);

  const addTags = React.useCallback(
    (raw: string) => {
      const tokens = raw
        .split(separators)
        .map((s) => s.trim())
        .filter(Boolean);
      if (tokens.length === 0) return;

      const existing = new Set(value);
      const next: string[] = [];
      for (const tok of tokens) {
        const clipped = maxLength > 0 ? tok.slice(0, maxLength) : tok;
        if (!clipped) continue;
        if (!allowDuplicates && existing.has(clipped)) continue;
        existing.add(clipped);
        next.push(clipped);
        if (max && value.length + next.length >= max) break;
      }
      if (next.length === 0) return;
      onChange([...value, ...next]);
      setDraft("");
    },
    [
      allowDuplicates,
      max,
      maxLength,
      onChange,
      separators,
      value,
    ],
  );

  const removeTag = (tag: string) => {
    if (disabled) return;
    onChange(value.filter((v) => v !== tag));
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (disabled) return;
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      if (draft.trim()) addTags(draft);
    } else if (e.key === "Backspace" && !draft && value.length > 0) {
      // 输入框为空时按 Backspace 删最后一个
      onChange(value.slice(0, -1));
    } else if (e.key === "Escape") {
      setDraft("");
    }
  };

  const handleBlur = () => {
    if (draft.trim()) addTags(draft);
  };

  const handlePaste = (e: React.ClipboardEvent<HTMLInputElement>) => {
    const pasted = e.clipboardData.getData("text");
    if (separators.test(pasted)) {
      e.preventDefault();
      addTags(pasted);
    }
  };

  return (
    <div
      className={cn(
        "flex flex-wrap items-center gap-1.5 min-h-9 w-full rounded-md border border-input bg-transparent px-2 py-1.5 text-sm shadow-sm transition-colors",
        "hover:border-foreground/40 focus-within:border-ring focus-within:ring-1 focus-within:ring-ring/40",
        disabled && "cursor-not-allowed opacity-50 bg-muted",
        className,
      )}
      onClick={() => inputRef.current?.focus()}
    >
      {value.map((tag) => (
        <Badge
          key={tag}
          variant="secondary"
          className="gap-1 px-1.5 py-0 text-[11px] font-normal"
        >
          <span className="truncate max-w-[160px]">{tag}</span>
          {!disabled && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                removeTag(tag);
              }}
              className="ml-0.5 rounded-sm hover:bg-foreground/10 p-0.5"
              aria-label={`remove ${tag}`}
            >
              <X className="h-3 w-3" />
            </button>
          )}
        </Badge>
      ))}
      <Input
        ref={inputRef}
        type="text"
        value={draft}
        disabled={disabled}
        placeholder={value.length === 0 ? effectivePlaceholder : undefined}
        onChange={(e) => {
          setDraft(e.target.value);
          onInputChange?.(e.target.value);
        }}
        onKeyDown={handleKeyDown}
        onBlur={handleBlur}
        onPaste={handlePaste}
        className="flex-1 min-w-[120px] h-7 border-0 bg-transparent px-1 shadow-none focus-visible:ring-0 focus-visible:ring-offset-0"
      />
    </div>
  );
}

TagInput.displayName = "TagInput";