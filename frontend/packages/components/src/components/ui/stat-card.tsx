//! StatCard / StatGrid · 统计卡片
//!
//! 匹配 design-system.css 的 .stat-grid / .stat-card 体系

import * as React from "react";
import { cn } from "../../lib/utils";
import { TrendingUp, TrendingDown } from "lucide-react";

export interface StatCardProps {
  label: string;
  value: string | number;
  icon?: React.ReactNode;
  trend?: number; // 正数表示上升，负数表示下降
  trendLabel?: string;
  className?: string;
}

export function StatCard({
  label,
  value,
  icon,
  trend,
  trendLabel,
  className,
}: StatCardProps) {
  const isPositive = trend !== undefined && trend >= 0;

  return (
    <div
      className={cn(
        "bg-card border border-border rounded-xl p-5 transition-colors hover:border-border-hover",
        className
      )}
    >
      <div className="flex items-center justify-between mb-3">
        {icon && (
          <div className="w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center text-lg">
            {icon}
          </div>
        )}
        {trend !== undefined && (
          <div
            className={cn(
              "inline-flex items-center gap-1 text-xs font-semibold px-2 py-0.5 rounded-full",
              isPositive
                ? "bg-success/10 text-success"
                : "bg-destructive/10 text-destructive"
            )}
          >
            {isPositive ? (
              <TrendingUp className="w-3 h-3" />
            ) : (
              <TrendingDown className="w-3 h-3" />
            )}
            {Math.abs(trend)}%
          </div>
        )}
      </div>

      <div
        className="text-3xl font-bold tracking-tight text-foreground leading-tight"
        style={{ fontFamily: "var(--font-display, inherit)" }}
      >
        {value}
      </div>

      <div className="text-xs text-muted-foreground mt-1">
        {label}
        {trendLabel && (
          <span className="text-muted-foreground/70"> · {trendLabel}</span>
        )}
      </div>
    </div>
  );
}

export interface StatGridProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
  columns?: 2 | 3 | 4;
}

export function StatGrid({
  className,
  children,
  columns = 4,
  ...props
}: StatGridProps) {
  const gridCols = {
    2: "grid-cols-1 sm:grid-cols-2",
    3: "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3",
    4: "grid-cols-1 sm:grid-cols-2 lg:grid-cols-4",
  };

  return (
    <div
      className={cn("grid gap-4 mb-6", gridCols[columns], className)}
      {...props}
    >
      {children}
    </div>
  );
}
