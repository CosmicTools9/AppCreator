/**
 * SchedulePanel · 日程管理工作区面板
 *
 * 右侧滑出面板内的日程操作工作区。
 * 基于 zc_id_plan（计划）+ zc_id_event（事件）双表模型展示日程项。
 * 包含视图切换（月历/周视图/列表）、迷你月历、周视图、
 * 时间轴、快速新建（含提醒设置）、待办清单、审批联动展示。
 */

import * as React from "react";
import { Calendar, Clock, List } from "lucide-react";
import { format } from "date-fns";

import { cn } from "../../lib/utils";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { ScrollArea } from "../ui/scroll-area";
import { Skeleton } from "../ui/skeleton";
import { EmptyState } from "../feedback/empty-state";
import { MiniCalendar } from "./MiniCalendar";
import { WeekView } from "./WeekView";
import { ScheduleTimeline } from "./ScheduleTimeline";
import { TodoList } from "./TodoList";
import { QuickAddForm } from "./QuickAddForm";
import type {
  SchedulePanelProps,
  ScheduleViewMode,
  ScheduleItem,
} from "./types";
import { useT } from "@alioth/i18n";
import { useDateFnsLocale } from "./useDateFnsLocale";

function useViewTabs() {
  const t = useT();
  return React.useMemo(() => [
    { id: "month" as ScheduleViewMode, label: t("components.schedule.view.month"), icon: Calendar },
    { id: "week" as ScheduleViewMode, label: t("components.schedule.view.week"), icon: Clock },
    { id: "list" as ScheduleViewMode, label: t("components.schedule.view.list"), icon: List },
  ], [t]);
}

/** 按选中日期筛选日程项（支持跨天区间） */
function filterItemsByDate(
  items: ScheduleItem[] | undefined,
  date: Date,
): ScheduleItem[] {
  const dateStr = format(date, "yyyy-MM-dd");
  return (items ?? []).filter((i) => {
    const start = i.span.dateStart;
    const end = i.span.dateEnd || start;
    if (!start || !end) return false;
    return dateStr >= start && dateStr <= end;
  });
}

/** 提取有日程的日期集合（展开跨天区间内的所有天） */
function extractItemDates(items: ScheduleItem[] | undefined): string[] {
  const dates = new Set<string>();
  (items ?? []).forEach((i) => {
    const start = i.span.dateStart;
    const end = i.span.dateEnd || start;
    if (!start) return;
    if (start === end || !end) {
      dates.add(start);
      return;
    }
    const startDate = new Date(start + "T00:00:00");
    const endDate = new Date(end + "T00:00:00");
    for (let d = new Date(startDate); d <= endDate; d.setDate(d.getDate() + 1)) {
      dates.add(
        `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`,
      );
    }
  });
  return Array.from(dates);
}

export const SchedulePanel = React.forwardRef<
  HTMLDivElement,
  SchedulePanelProps
>(
  (
    {
      items = [],
      todos = [],
      viewMode: controlledViewMode,
      onViewModeChange,
      onSelectDate,
      onAddItem,
      onToggleTodo,
      onItemClick,
      onApprovalClick,
      loading = false,
      selectedDate: controlledSelectedDate,
      className,
    },
    ref,
  ) => {
    const t = useT();
    const viewTabs = useViewTabs();
    const [internalViewMode, setInternalViewMode] =
      React.useState<ScheduleViewMode>("month");
    const [internalSelectedDate, setInternalSelectedDate] =
      React.useState<Date>(controlledSelectedDate ?? new Date());

    const viewMode = controlledViewMode ?? internalViewMode;
    const selectedDate = controlledSelectedDate ?? internalSelectedDate;

    const setViewMode = (mode: ScheduleViewMode) => {
      setInternalViewMode(mode);
      onViewModeChange?.(mode);
    };

    const setSelectedDate = (date: Date) => {
      setInternalSelectedDate(date);
      onSelectDate?.(date);
    };

    // 同步外部 controlled 值
    React.useEffect(() => {
      if (controlledSelectedDate) {
        setInternalSelectedDate(controlledSelectedDate);
      }
    }, [controlledSelectedDate]);

    const filteredItems = React.useMemo(
      () => filterItemsByDate(items, selectedDate),
      [items, selectedDate],
    );

    const itemDates = React.useMemo(
      () => extractItemDates(items),
      [items],
    );

    const selectedStr = format(
      selectedDate,
      t("components.schedule.format.fullDate", {}, { fallback: "yyyy-MM-dd" }),
      { locale: useDateFnsLocale() },
    );

    return (
      <div ref={ref} className={cn("flex h-full flex-col", className)}>
        {/* View Mode Tabs */}
        <div className="px-4 pt-3 pb-2">
          <Tabs
            value={viewMode}
            onValueChange={(v) => setViewMode(v as ScheduleViewMode)}
            className="w-full"
          >
            <TabsList className="w-full grid grid-cols-3 h-9 p-1 bg-muted">
              {viewTabs.map((tab) => (
                <TabsTrigger
                  key={tab.id}
                  value={tab.id}
                  className="text-xs font-medium flex items-center gap-1.5"
                >
                  <tab.icon className="w-3.5 h-3.5" />
                  {tab.label}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </div>

        {/* Scrollable Content */}
        <div className="flex-1 overflow-hidden">
          <ScrollArea className="h-full px-4 py-2">
            {loading ? (
              <div className="space-y-4">
                <Skeleton className="h-64 w-full rounded-xl" />
                <Skeleton className="h-12 w-full rounded-xl" />
                <Skeleton className="h-52 w-full rounded-xl" />
              </div>
            ) : (
              <div className="space-y-4 pb-4">
              {/* ===== Month View ===== */}
              {viewMode === "month" && (
                <>
                  <MiniCalendar
                    selectedDate={selectedDate}
                    onSelectDate={setSelectedDate}
                    eventDates={itemDates}
                  />

                  <QuickAddForm
                    onAdd={onAddItem}
                    defaultDate={format(selectedDate, "yyyy-MM-dd")}
                  />

                  {/* Today's Timeline */}
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
                        <Clock className="w-4 h-4 text-primary" />
                        {format(selectedDate, t("components.schedule.format.monthDay", {}, { fallback: "MM-dd" }))} {t("components.schedule.title.withCount", { count: String(filteredItems.length) })}
                      </h3>
                      <span className="text-xs text-muted-foreground">
                        {t("components.schedule.completedRatio", { completed: String(filteredItems.filter((i) => i.done).length), total: String(filteredItems.length) })}
                      </span>
                    </div>

                    {filteredItems.length > 0 ? (
                      <ScheduleTimeline
                        items={filteredItems}
                        onItemClick={onItemClick}
                        onApprovalClick={onApprovalClick}
                      />
                    ) : (
                      <EmptyState
                        icon={<Calendar className="h-10 w-10" />}
                        title={t("components.schedule.empty.noEvents")}
                        description={t("components.schedule.empty.noEventsDesc", { date: selectedStr })}
                      />
                    )}
                  </div>

                  <TodoList items={todos} onToggle={onToggleTodo} />
                </>
              )}

              {/* ===== Week View ===== */}
              {viewMode === "week" && (
                <>
                  <WeekView
                    baseDate={selectedDate}
                    items={items}
                    onSelectDate={setSelectedDate}
                    onItemClick={onItemClick}
                  />

                  <QuickAddForm
                    onAdd={onAddItem}
                    defaultDate={format(selectedDate, "yyyy-MM-dd")}
                  />

                  <TodoList items={todos} onToggle={onToggleTodo} />
                </>
              )}

              {/* ===== List View ===== */}
              {viewMode === "list" && (
                <>
                  <QuickAddForm
                    onAdd={onAddItem}
                    defaultDate={format(selectedDate, "yyyy-MM-dd")}
                  />

                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
                        <Clock className="w-4 h-4 text-primary" />
                        {t("components.schedule.allEvents", { count: String(items.length) })}
                      </h3>
                      <span className="text-xs text-muted-foreground">
                        {t("components.schedule.completedRatio", { completed: String(items.filter((i) => i.done).length), total: String(items.length) })}
                      </span>
                    </div>

                    {items.length > 0 ? (
                      <ScheduleTimeline
                        items={items}
                        onItemClick={onItemClick}
                        onApprovalClick={onApprovalClick}
                      />
                    ) : (
                      <EmptyState
                        icon={<Calendar className="h-10 w-10" />}
                        title={t("components.schedule.empty.noEvents")}
                        description={t("components.schedule.empty.noEventsAtAll")}
                      />
                    )}
                  </div>

                  <TodoList items={todos} onToggle={onToggleTodo} />
                </>
              )}
            </div>
          )}
          </ScrollArea>
        </div>
      </div>
    );
  },
);

SchedulePanel.displayName = "SchedulePanel";
