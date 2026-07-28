import * as React from "react";
import { cn } from "../../lib/utils";

export interface ActivityItemProps {
  title: string;
  description?: string;
  timestamp?: string;
  status?: "default" | "success" | "warning" | "error" | "info";
  avatar?: React.ReactNode;
  onClick?: () => void;
  className?: string;
}

const statusVariants = {
  default: "bg-muted text-muted-foreground",
  success: "bg-success/20 text-success",
  warning: "bg-warning/20 text-warning",
  error: "bg-destructive/20 text-destructive",
  info: "bg-info/20 text-info",
};

export function ActivityItem({
  title,
  description,
  timestamp,
  status = "default",
  avatar,
  onClick,
  className,
}: ActivityItemProps) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "flex items-start gap-3 p-3 rounded-lg hover:bg-muted/50 transition-colors cursor-pointer",
        className,
      )}
    >
      {avatar ? (
        <div className="shrink-0">{avatar}</div>
      ) : (
        <div
          className={cn(
            "w-2 h-2 rounded-full mt-2 shrink-0",
            statusVariants[status],
          )}
        />
      )}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground truncate">{title}</p>
        {description && (
          <p className="text-xs text-muted-foreground mt-0.5 truncate">
            {description}
          </p>
        )}
        {timestamp && (
          <p className="text-xs text-muted-foreground/70 mt-1">{timestamp}</p>
        )}
      </div>
    </div>
  );
}
