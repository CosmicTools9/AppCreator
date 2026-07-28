//! DeviceStatus · 设备状态指示器（带脉冲动画）
//!
//! 匹配 design-system.css 的 .device-status-online / .device-status-offline

import * as React from "react";
import { cn } from "../../lib/utils";

export interface DeviceStatusProps extends React.HTMLAttributes<HTMLDivElement> {
  status: "online" | "offline" | "warning" | "error";
  label?: string;
  showPulse?: boolean;
}

export function DeviceStatus({
  status,
  label,
  showPulse = true,
  className,
  ...props
}: DeviceStatusProps) {
  const config = {
    online:  { dot: "bg-success", pulse: "bg-success/60", text: "text-success" },
    offline: { dot: "bg-muted-foreground", pulse: undefined, text: "text-muted-foreground" },
    warning: { dot: "bg-warning", pulse: "bg-warning/60", text: "text-warning" },
    error:   { dot: "bg-destructive", pulse: "bg-destructive/60", text: "text-destructive" },
  };

  const c = config[status];
  const displayLabel = label || (status === "online" ? "在线" : status === "offline" ? "离线" : status === "warning" ? "告警" : "故障");

  return (
    <div className={cn("inline-flex items-center gap-2", className)} {...props}>
      <span className="relative flex h-2.5 w-2.5">
        {showPulse && c.pulse && (
          <span
            className={cn(
              "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
              c.pulse
            )}
          />
        )}
        <span className={cn("relative inline-flex rounded-full h-2.5 w-2.5", c.dot)} />
      </span>
      <span className={cn("text-xs font-medium", c.text)}>{displayLabel}</span>
    </div>
  );
}
