//! ProgressBar · 进度条（带 warning/success 变体）
//!
//! 匹配 design-system.css 的 .progress-bar 体系

import * as React from "react";
import { cn } from "../../lib/utils";

export interface ProgressBarProps extends React.HTMLAttributes<HTMLDivElement> {
  value: number; // 0-100
  max?: number;
  variant?: "default" | "success" | "warning";
  size?: "sm" | "md";
  showLabel?: boolean;
  label?: string;
}

export function ProgressBar({
  value,
  max = 100,
  variant = "default",
  size = "md",
  showLabel = true,
  label,
  className,
  ...props
}: ProgressBarProps) {
  const pct = Math.min(100, Math.max(0, (value / max) * 100));

  const trackClasses = {
    sm: "h-1.5",
    md: "h-2",
  };

  const fillClasses = {
    default: "bg-primary",
    success: "bg-success",
    warning: "bg-warning",
  };

  return (
    <div className={cn("w-full", className)} {...props}>
      {(showLabel || label) && (
        <div className="flex items-center justify-between mb-1.5">
          {label && (
            <span className="text-xs text-muted-foreground">{label}</span>
          )}
          <span className="text-xs font-medium text-foreground tabular-nums">
            {Math.round(pct)}%
          </span>
        </div>
      )}
      <div
        className={cn(
          "w-full rounded-full bg-muted overflow-hidden",
          trackClasses[size]
        )}
      >
        <div
          className={cn(
            "h-full rounded-full transition-all duration-500 ease-out",
            fillClasses[variant]
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
