import * as React from "react";
import { TrendingUp, TrendingDown } from "lucide-react";
import { cn } from "../../lib/utils";

export interface StatCardProps {
  label: string;
  value: string | number;
  unit?: string;
  change?: number | string;
  trend?: "up" | "down" | "neutral";
  icon: React.ComponentType<{ className?: string }>;
  onClick?: () => void;
  className?: string;
}

export function StatCard({
  label,
  value,
  unit,
  change,
  trend,
  icon: Icon,
  onClick,
  className,
}: StatCardProps) {
  const isUp =
    trend === "up" || (trend === undefined && typeof change === "number" && change > 0);
  const isDown =
    trend === "down" || (trend === undefined && typeof change === "number" && change < 0);
  const hasChange = change !== undefined;

  return (
    <div
      onClick={onClick}
      className={cn(
        "bg-card rounded-xl border p-5 hover:shadow-sm transition-shadow",
        onClick && "cursor-pointer",
        className,
      )}
    >
      <div className="flex items-start justify-between">
        <div className="p-2 rounded-lg bg-primary/10 text-primary">
          <Icon className="w-5 h-5" />
        </div>
        {hasChange && (
          <span
            className={cn(
              "inline-flex items-center gap-0.5 text-xs font-medium px-2.5 py-1 rounded-full",
              isUp &&
                "bg-success/20 text-success",
              isDown &&
                "bg-destructive/20 text-destructive",
              !isUp && !isDown && "bg-muted text-muted-foreground",
            )}
          >
            {isUp && <TrendingUp className="w-3 h-3" />}
            {isDown && <TrendingDown className="w-3 h-3" />}
            {typeof change === "number" ? `${Math.abs(change)}%` : change}
          </span>
        )}
      </div>
      <div className="mt-4">
        <p className="text-2xl font-bold text-foreground tracking-tight">
          {value}
          {unit && (
            <span className="text-sm font-medium text-muted-foreground gl-1.5">
              {unit}
            </span>
          )}
        </p>
        <p className="text-sm text-muted-foreground mt-1">{label}</p>
      </div>
    </div>
  );
}
