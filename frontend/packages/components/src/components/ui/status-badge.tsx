//! StatusBadge · 状态徽章（带圆点前缀）
//!
//! 匹配 design-system.css 的 .status-badge 体系。
//! 支持两种模式：
//!   1. variant prop（语义变体：active/draft/pending/archived/rejected）
//!   2. token prop（从 STATUS_COLOR_TOKENS 取色，适用于自定义业务状态）

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/utils";
import { STATUS_COLOR_TOKENS } from "../../tokens/colors";

const statusBadgeVariants = cva(
  "inline-flex items-center gap-1 rounded-full border text-xs font-medium px-2.5 py-0.5",
  {
    variants: {
      variant: {
        active:   "bg-success/10 text-success border-success/20",
        draft:    "bg-info/10 text-info border-info/20",
        pending:  "bg-warning/10 text-warning border-warning/20",
        archived: "bg-muted text-muted-foreground border-border",
        rejected: "bg-destructive/10 text-destructive border-destructive/20",
      },
    },
    defaultVariants: {
      variant: "active",
    },
  }
);

export type BadgeVariant = VariantProps<typeof statusBadgeVariants>["variant"];

export interface StatusBadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof statusBadgeVariants> {
  label?: string;
  /** STATUS_COLOR_TOKENS 中的 token 名，覆盖 variant */
  token?: string;
}

export function StatusBadge({
  className,
  variant,
  token,
  label,
  children,
  ...props
}: StatusBadgeProps) {
  const dotColorMap: Record<string, string> = {
    active:   "bg-success",
    draft:    "bg-info",
    pending:  "bg-warning",
    archived: "bg-muted-foreground",
    rejected: "bg-destructive",
  };

  // token mode: 从 STATUS_COLOR_TOKENS 取色
  if (token && STATUS_COLOR_TOKENS[token]) {
    const entry = STATUS_COLOR_TOKENS[token];
    return (
      <span
        className={cn(entry.badge, className)}
        {...props}
      >
        <span
          className={cn("w-1.5 h-1.5 rounded-full flex-shrink-0", entry.dot)}
        />
        {label ?? children}
      </span>
    );
  }

  // variant mode: 原生语义变体
  const dotClass = dotColorMap[variant || "active"];

  return (
    <span
      className={cn(statusBadgeVariants({ variant }), className)}
      {...props}
    >
      <span
        className={cn("w-1.5 h-1.5 rounded-full flex-shrink-0", dotClass)}
      />
      {label ?? children}
    </span>
  );
}
