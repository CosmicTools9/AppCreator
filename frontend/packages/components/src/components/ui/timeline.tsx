//! Timeline · 变更历史时间线
//!
//! 匹配 design-system.css 的 .timeline 体系

import * as React from "react";
import { cn } from "../../lib/utils";

export interface TimelineItem {
  id: string | number;
  time: string;
  title: string;
  description?: string;
  status?: "completed" | "active" | "pending";
}

export interface TimelineProps extends React.HTMLAttributes<HTMLDivElement> {
  items: TimelineItem[];
}

export function Timeline({ className, items, ...props }: TimelineProps) {
  return (
    <div className={cn("relative pl-4", className)} {...props}>
      {/* 竖线 */}
      <div className="absolute top-2 bottom-2 w-px bg-border" style={{ left: 7 }} />

      <div className="space-y-4">
        {items.map((item, index) => {
          const isLast = index === items.length - 1;
          const isActive = item.status === "active";
          const isCompleted = item.status === "completed" || (!isActive && !isLast);

          return (
            <div key={item.id} className="relative flex gap-3">
              {/* 圆点 */}
              <div
                className={cn(
                  "relative z-10 mt-1.5 w-2 h-2 rounded-full flex-shrink-0 ring-4",
                  isActive
                    ? "bg-primary ring-primary/10"
                    : isCompleted
                    ? "bg-muted-foreground ring-background"
                    : "bg-border ring-background"
                )}
              />

              {/* 内容 */}
              <div className="flex-1 min-w-0">
                <div className="text-xs text-muted-foreground font-mono">
                  {item.time}
                </div>
                <div className="text-sm font-medium text-foreground mt-0.5">
                  {item.title}
                </div>
                {item.description && (
                  <div className="text-xs text-muted-foreground mt-0.5 leading-relaxed">
                    {item.description}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
