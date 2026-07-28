//! FilterPill · 筛选胶囊按钮
//!
//! 匹配 design-system.css 的 .filter-pill

import * as React from "react";
import { ChevronDown } from "lucide-react";
import { cn } from "../../lib/utils";

export interface FilterPillProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  label: string;
  active?: boolean;
}

export function FilterPill({
  className,
  label,
  active,
  ...props
}: FilterPillProps) {
  return (
    <button
      type="button"
      className={cn(
        "inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium border transition-colors",
        "bg-card text-foreground border-border hover:border-border-hover hover:bg-muted",
        active && "border-primary text-primary bg-primary/5",
        className
      )}
      {...props}
    >
      {label}
      <ChevronDown className="w-3 h-3 opacity-60" />
    </button>
  );
}
