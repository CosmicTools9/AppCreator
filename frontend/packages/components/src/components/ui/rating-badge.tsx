//! RatingBadge · 评级徽章（A/AP/BP/B 等信用评级）
//!
//! 匹配 design-system.css 的 .rating-badge 体系

import * as React from "react";
import { cn } from "../../lib/utils";

export type RatingLevel = "a" | "ap" | "bp" | "b" | "c" | "d";

export interface RatingBadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  level: RatingLevel | string;
  size?: "sm" | "md";
}

const ratingConfig: Record<string, { label: string; className: string }> = {
  a:   { label: "A",   className: "bg-success/10 text-success border-success/20" },
  ap:  { label: "A+",  className: "bg-success/20 text-success border-success/30" },
  bp:  { label: "B+",  className: "bg-info/10 text-info border-info/20" },
  b:   { label: "B",   className: "bg-info/10 text-info border-info/20" },
  c:   { label: "C",   className: "bg-warning/10 text-warning border-warning/20" },
  d:   { label: "D",   className: "bg-destructive/10 text-destructive border-destructive/20" },
};

export function RatingBadge({
  level,
  size = "sm",
  className,
  ...props
}: RatingBadgeProps) {
  const normalized = level.toLowerCase().replace(/[^a-z]/g, "") as RatingLevel;
  const config = ratingConfig[normalized] || ratingConfig["c"];

  const sizeClasses = {
    sm: "text-xs px-1.5 py-0.5",
    md: "text-xs px-2.5 py-1",
  };

  return (
    <span
      className={cn(
        "inline-flex items-center justify-center rounded-md border font-bold",
        sizeClasses[size],
        config.className,
        className
      )}
      {...props}
    >
      {config.label}
    </span>
  );
}
