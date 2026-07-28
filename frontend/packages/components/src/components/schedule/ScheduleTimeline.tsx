/**
 * ScheduleTimeline · 日程时间轴
 *
 * 按时间顺序展示日程项列表（ScheduleItem）。
 * 支持完成状态、类型标签、地点、参与主体、关联审批联动展示。
 * 符合 Gateway 设计规范 §4.6 日程时间轴项规范。
 */

import * as React from "react";
import { Clock, MapPin, FileCheck } from "lucide-react";
import { cn } from "../../lib/utils";
import type { ScheduleTimelineProps, ScheduleItem } from "./types";
import { useT } from "@alioth/i18n";

/** 事件类型标签映射 */
function useItemTypeLabels(): Record<ScheduleItem["type"], string> {
  const t = useT();
  return {
    meeting: t("components.schedule.type.meeting"),
    sync: t("components.schedule.type.sync"),
    client: t("components.schedule.type.client"),
    development: t("components.schedule.type.development"),
    dev: t("components.schedule.type.dev"),
    team: t("components.schedule.type.team"),
    review: t("components.schedule.type.review"),
    personal: t("components.schedule.type.personal"),
    other: t("components.schedule.type.other"),
  };
}

/** 事件类型颜色（适配深浅主题） */
const itemTypeColors: Record<ScheduleItem["type"], string> = {
  meeting: "bg-primary/10 dark:bg-primary/20 text-primary ",
  sync: "bg-info/10 dark:bg-info/20 text-info ",
  client: "bg-warning/10 dark:bg-warning/20 text-warning ",
  development: "bg-info/10 dark:bg-info/20 text-info ",
  dev: "bg-info/10 dark:bg-info/20 text-info ",
  team: "bg-success/10 dark:bg-success/20 text-success ",
  review: "bg-destructive/10 dark:bg-destructive/20 text-destructive ",
  personal: "bg-muted/10 text-muted-foreground ",
  other: "bg-muted/10 text-muted-foreground ",
};

/** 审批状态样式（适配深浅主题） */
const approvalStatusStyles = {
  pending: "bg-warning/10 dark:bg-warning/20 text-warning  border-warning/20",
  approved: "bg-success/10 dark:bg-success/20 text-success  border-success/20",
  rejected: "bg-destructive/10 dark:bg-destructive/20 text-destructive  border-destructive/20",
};

function useApprovalStatusLabels(): Record<string, string> {
  const t = useT();
  return {
    pending: t("components.schedule.approval.pending"),
    approved: t("components.schedule.approval.approved"),
    rejected: t("components.schedule.approval.rejected"),
  };
}

/** 单个时间轴项 */
function TimelineItem({
  item,
  onClick,
  onApprovalClick,
}: {
  item: ScheduleItem;
  onClick?: (item: ScheduleItem) => void;
  onApprovalClick?: (approval: NonNullable<ScheduleItem["linkedApproval"]>)
    => void;
}) {
  const t = useT();
  const itemTypeLabels = useItemTypeLabels();
  const approvalStatusLabels = useApprovalStatusLabels();
  return (
    <div
      className={cn(
        "flex items-start gap-3 p-3 rounded-xl border transition-colors",
        item.done
          ? "bg-muted/50 border-border/50"
          : "bg-card border-border hover:border-primary/30 cursor-pointer",
      )}
      onClick={() => onClick?.(item)}
    >
      {/* Time Column */}
      <div className="w-14 text-center shrink-0">
        <p
          className={cn(
            "text-sm font-bold",
            item.done ? "text-muted-foreground" : "text-foreground",
          )}
        >
          {item.span.timeStart || "--:--"}
        </p>
        {item.span.timeEnd && item.span.timeEnd !== item.span.timeStart && (
          <p className="text-xs text-muted-foreground">{item.span.timeEnd}</p>
        )}
        <p className="text-xs text-muted-foreground">{item.duration}</p>
      </div>

      {/* Divider */}
      <div
        className={cn(
          "w-0.5 self-stretch rounded-full",
          item.done ? "bg-border" : "bg-primary",
        )}
      />

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <p
            className={cn(
              "text-sm font-semibold truncate",
              item.done ? "text-muted-foreground line-through" : "text-foreground",
            )}
          >
            {item.title}
          </p>
          {item.cron && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-primary/10 dark:bg-primary/20 text-primary">
              {t("components.schedule.reminder.recurring")}
            </span>
          )}
        </div>

        {/* Meta Row */}
        <div className="flex items-center flex-wrap gap-2 text-xs text-muted-foreground">
          <span
            className={cn(
              "text-xs px-1.5 py-0.5 rounded",
              itemTypeColors[item.type],
            )}
          >
            {itemTypeLabels[item.type]}
          </span>

          {item.location && (
            <span className="flex items-center gap-1">
              <MapPin className="w-3 h-3 shrink-0" />
              <span className="truncate">{item.location}</span>
            </span>
          )}

          {item.subject && (
            <span className="truncate text-xs text-muted-foreground/70">
              @{item.subject}
            </span>
          )}

          {item.reminder && item.reminder.offset > 0 && (
            <span className="flex items-center gap-1 text-xs text-primary bg-primary/5 px-1.5 py-0.5 rounded">
              <Clock className="w-3 h-3" />
              {item.reminder.offset < 60
                ? t("components.schedule.reminder.minutesBefore", { offset: String(item.reminder.offset) })
                : item.reminder.offset === 60
                  ? t("components.schedule.reminder.hourBefore")
                  : t("components.schedule.reminder.dayBefore")}
            </span>
          )}

          {/* 关联审批（审批联动） */}
          {item.linkedApproval && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onApprovalClick?.(item.linkedApproval!);
              }}
              className={cn(
                "inline-flex items-center gap-1 px-2 py-0.5 rounded-md border text-xs font-medium transition-colors cursor-pointer hover:opacity-80",
                approvalStatusStyles[item.linkedApproval.status],
              )}
            >
              <FileCheck className="w-3 h-3" />
              {approvalStatusLabels[item.linkedApproval.status]}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export const ScheduleTimeline = React.forwardRef<
  HTMLDivElement,
  ScheduleTimelineProps
>(({ items, onItemClick, onApprovalClick, className }, ref) => {
  const sortedItems = React.useMemo(
    () =>
      [...(items ?? [])].sort((a, b) =>
        (a.span.timeStart || "99:99").localeCompare(b.span.timeStart || "99:99"),
      ),
    [items],
  );

  return (
    <div ref={ref} className={cn("space-y-2", className)}>
      {sortedItems.map((item) => (
        <TimelineItem
          key={item.id}
          item={item}
          onClick={onItemClick}
          onApprovalClick={onApprovalClick}
        />
      ))}
    </div>
  );
});

ScheduleTimeline.displayName = "ScheduleTimeline";
