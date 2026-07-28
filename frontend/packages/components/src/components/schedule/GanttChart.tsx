/**
 * GanttChart · 甘特图
 *
 * 跨模块通用的时间排程可视化组件。
 * 支持按 plan_type 分组、进度填充、冲突标记、资源名称解析。
 * 适用于 plan / project / transport 等需要时间轴展示的模块。
 */

import * as React from "react";
import { useT } from "@alioth/i18n";
import { AlertTriangle } from "lucide-react";
import type { GanttItem, GanttChartProps } from "./types";

const PLAN_TYPE_LABEL_KEY: Record<string, string> = {
  purchase: "plan.panorama.purchasePlans",
  inbound: "plan.panorama.inboundPlans",
  material: "plan.panorama.materialPlans",
  making: "plan.panorama.makingPlans",
  outbound: "plan.panorama.outboundPlans",
  delivery: "plan.panorama.deliveryPlans",
  perform: "plan.panorama.performPlans",
  project: "plan.panorama.projectPlans",
  promotion: "plan.panorama.promotionPlans",
  recruitment: "plan.panorama.recruitmentPlans",
};

const ROW_HEIGHT = 36;
const HEADER_HEIGHT = 40;
const LEFT_WIDTH = 180;
const DAY_WIDTH = 40;

function formatDate(d: string | Date, locale = "zh-CN"): string {
  const date = typeof d === "string" ? new Date(d) : d;
  return new Intl.DateTimeFormat(locale, { month: "numeric", day: "numeric" }).format(date);
}

function daysBetween(a: Date, b: Date): number {
  return (b.getTime() - a.getTime()) / (1000 * 60 * 60 * 24);
}

function detectConflicts(items: GanttItem[]): Set<string | number> {
  const conflicts = new Set<string | number>();

  function scanByResource(
    getKey: (item: GanttItem) => string | number | null | undefined,
    _resourceType: string
  ) {
    const groups = new Map<string | number, GanttItem[]>();
    for (const item of items) {
      const key = getKey(item);
      if (key == null) continue;
      const arr = groups.get(key) ?? [];
      arr.push(item);
      groups.set(key, arr);
    }

    for (const groupItems of groups.values()) {
      groupItems.sort(
        (a, b) => new Date(a.start_date).getTime() - new Date(b.start_date).getTime()
      );
      for (let i = 1; i < groupItems.length; i++) {
        const prev = groupItems[i - 1];
        const curr = groupItems[i];
        const prevEnd = new Date(prev.end_date).getTime();
        const currStart = new Date(curr.start_date).getTime();
        if (currStart < prevEnd) {
          conflicts.add(prev.id);
          conflicts.add(curr.id);
        }
      }
    }
  }

  scanByResource((item) => item.fk_place, "place");
  scanByResource((item) => item.fk_subject, "subject");

  return conflicts;
}

export const GanttChart = React.memo(function GanttChart({
  data,
}: GanttChartProps): React.ReactElement {
  const t = useT();

  const rangeStart = React.useMemo(() => new Date(data.range_start), [data.range_start]);
  const rangeEnd = React.useMemo(() => new Date(data.range_end), [data.range_end]);
  const totalDays = React.useMemo(
    () => Math.max(1, daysBetween(rangeStart, rangeEnd)),
    [rangeStart, rangeEnd]
  );

  const groups = React.useMemo(() => {
    const map = new Map<string, GanttItem[]>();
    for (const item of data.items) {
      if (!map.has(item.plan_type)) map.set(item.plan_type, []);
      map.get(item.plan_type)!.push(item);
    }
    return Array.from(map.entries()).map(([plan_type, items]) => ({
      plan_type,
      label: t(PLAN_TYPE_LABEL_KEY[plan_type] || plan_type, {}, { fallback: plan_type }),
      color: items[0]?.color || "#6B7280",
      items,
    }));
  }, [data.items, t]);

  const allConflicts = React.useMemo(() => detectConflicts(data.items), [data.items]);

  const ticks = React.useMemo(() => {
    const arr: Date[] = [];
    let d = new Date(rangeStart);
    while (d <= rangeEnd) {
      arr.push(new Date(d));
      d.setDate(d.getDate() + 7);
    }
    return arr;
  }, [rangeStart, rangeEnd]);

  const timelineWidth = Math.max(600, totalDays * DAY_WIDTH);

  return (
    <div className="rounded-xl border border-border bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-border">
        <div className="text-base font-semibold text-foreground">
          {t("plan.panorama.ganttTitle", {}, { fallback: "Plan Schedule Gantt Chart" })}
        </div>
        {allConflicts.size > 0 && (
          <div className="flex items-center gap-1.5 text-xs text-destructive">
            <AlertTriangle size={14} />
            {t(
              "plan.panorama.conflictsDetected",
              { count: allConflicts.size },
              { fallback: `${allConflicts.size} time conflicts detected` }
            )}
          </div>
        )}
      </div>

      <div className="flex overflow-auto">
        {/* Left name column */}
        <div style={{ width: LEFT_WIDTH, flexShrink: 0 }} className="border-r border-border">
          <div style={{ height: HEADER_HEIGHT }} className="border-b border-border bg-muted/30" />
          {groups.map((g) => (
            <React.Fragment key={g.plan_type}>
              <div
                style={{ height: ROW_HEIGHT }}
                className="flex items-center px-3 text-xs font-semibold text-muted-foreground bg-muted/30 border-b border-border"
              >
                <div
                  className="rounded-full mr-2"
                  style={{ width: 8, height: 8, background: g.color }}
                />
                {g.label}
              </div>
              {g.items.map((item) => (
                <div
                  key={item.id}
                  style={{ height: ROW_HEIGHT }}
                  className="flex items-center px-3 text-xs text-muted-foreground border-b border-border/50 truncate"
                  title={item.name}
                >
                  {allConflicts.has(item.id) && (
                    <AlertTriangle size={12} className="text-destructive mr-1 flex-shrink-0" />
                  )}
                  {item.name}
                </div>
              ))}
            </React.Fragment>
          ))}
        </div>

        {/* Right timeline */}
        <div style={{ position: "relative", width: timelineWidth, minWidth: timelineWidth }}>
          {/* Date ticks */}
          <div
            style={{ height: HEADER_HEIGHT }}
            className="relative border-b border-border bg-muted/30"
          >
            {ticks.map((tick, i) => {
              const left = (daysBetween(rangeStart, tick) / totalDays) * timelineWidth;
              return (
                <div
                  key={i}
                  style={{ position: "absolute", left, top: 0, bottom: 0 }}
                  className="flex items-center pl-1 text-xs text-muted-foreground border-l border-dashed border-border"
                >
                  {formatDate(tick)}
                </div>
              );
            })}
            <TodayMarker
              rangeStart={rangeStart}
              rangeEnd={rangeEnd}
              timelineWidth={timelineWidth}
            />
          </div>

          {/* Bars */}
          {groups.map((g) => (
            <React.Fragment key={g.plan_type}>
              <div
                style={{ height: ROW_HEIGHT }}
                className="border-b border-border bg-muted/20"
              />
              {g.items.map((item) => {
                const start = new Date(item.start_date);
                const end = new Date(item.end_date);
                const left = (daysBetween(rangeStart, start) / totalDays) * timelineWidth;
                const width = Math.max(2, (daysBetween(start, end) / totalDays) * timelineWidth);
                const progress =
                  item.progress_pct != null
                    ? Math.min(100, Math.max(0, item.progress_pct))
                    : 0;
                const isConflict = allConflicts.has(item.id);

                return (
                  <div
                    key={item.id}
                    style={{ height: ROW_HEIGHT }}
                    className="relative border-b border-border/50"
                  >
                    <div
                      style={{
                        position: "absolute",
                        left: Math.max(0, left),
                        width: Math.min(timelineWidth - Math.max(0, left), width),
                        top: 8,
                        height: 20,
                        borderRadius: 4,
                        background: `${item.color}20`,
                        border: `1px solid ${isConflict ? "#EF4444" : `${item.color}50`}`,
                        overflow: "hidden",
                      }}
                      title={`${item.name}: ${formatDate(start)} → ${formatDate(end)} (${progress.toFixed(0)}%)\n${
                        item.fk_place_name ? `场所: ${item.fk_place_name}\n` : ""
                      }${item.fk_subject_name ? `主体: ${item.fk_subject_name}` : ""}`}
                    >
                      <div
                        style={{
                          width: `${progress}%`,
                          height: "100%",
                          background: item.color,
                          opacity: 0.7,
                        }}
                      />
                    </div>
                  </div>
                );
              })}
            </React.Fragment>
          ))}
        </div>
      </div>
    </div>
  );
});

GanttChart.displayName = "GanttChart";

function TodayMarker({
  rangeStart,
  rangeEnd,
  timelineWidth,
}: {
  rangeStart: Date;
  rangeEnd: Date;
  timelineWidth: number;
}): React.ReactElement | null {
  const t = useT();
  const now = new Date();
  if (now < rangeStart || now > rangeEnd) return null;
  const left =
    (daysBetween(rangeStart, now) / daysBetween(rangeStart, rangeEnd)) * timelineWidth;
  return (
    <div
      style={{ position: "absolute", left, top: 0, bottom: 0, width: 2, zIndex: 10 }}
      className="bg-destructive"
    >
      <div className="absolute top-0.5 -left-3.5 text-[10px] font-bold text-destructive bg-background px-0.5 rounded">
        {t("plan.common.today", {}, { fallback: "Today" })}
      </div>
    </div>
  );
}
