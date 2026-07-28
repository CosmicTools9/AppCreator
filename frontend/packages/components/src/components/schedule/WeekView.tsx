/**
 * WeekView · 周视图
 *
 * 横向展示一周 7 天的日程概览。
 * 每天一列，顶部显示日期和星期，下方罗列当日日程项（ScheduleItem）。
 */

import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import {
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  format,
  isSameDay,
  addWeeks,
  subWeeks,
  isToday,
} from "date-fns";

import { cn } from "../../lib/utils";
import { ScrollArea } from "../ui/scroll-area";
import type { WeekViewProps, ScheduleItem } from "./types";
import { useT } from "@alioth/i18n";
import { useDateFnsLocale } from "./useDateFnsLocale";

/** 日程类型对应的颜色 */
function getItemTypeColor(type: ScheduleItem["type"]) {
  const map: Record<string, string> = {
    meeting: "bg-primary/10 dark:bg-primary/20 text-primary border-primary/10",
    sync: "bg-info/10 dark:bg-info/20 text-info border-info/10",
    client: "bg-warning/10 dark:bg-warning/20 text-warning border-warning/10",
    development: "bg-info/10 dark:bg-info/20 text-info border-info/10",
    team: "bg-success/10 dark:bg-success/20 text-success border-success/10",
    review: "bg-destructive/10 dark:bg-destructive/20 text-destructive border-destructive/10",
    personal: "bg-muted text-muted-foreground border-muted",
    other: "bg-muted text-muted-foreground border-muted",
  };
  return map[type] || map.other;
}

export const WeekView = React.forwardRef<HTMLDivElement, WeekViewProps>(
  ({ baseDate = new Date(), items, onSelectDate, onItemClick, className }, _ref) => {
    const t = useT();
    const dateLocale = useDateFnsLocale();
    const [currentWeek, setCurrentWeek] = React.useState(baseDate);

    React.useEffect(() => {
      setCurrentWeek(baseDate);
    }, [baseDate]);

    const weekDays = React.useMemo(() => {
      const start = startOfWeek(currentWeek, { weekStartsOn: 1 });
      const end = endOfWeek(currentWeek, { weekStartsOn: 1 });
      return eachDayOfInterval({ start, end });
    }, [currentWeek]);

    const weekRangeText = React.useMemo(() => {
      const start = weekDays[0];
      const end = weekDays[6];
      const sameMonth = start.getMonth() === end.getMonth();
      if (sameMonth) {
        return `${format(start, t("components.schedule.format.yearMonth", {}, { fallback: "yyyy-MM" }), { locale: dateLocale })} ${format(start, "d")}–${format(end, "d")}${t("components.schedule.week.daySuffix")}`;
      }
      return `${format(start, t("components.schedule.format.monthDay", {}, { fallback: "MM-dd" }), { locale: dateLocale })} – ${format(end, t("components.schedule.format.monthDay", {}, { fallback: "MM-dd" }), { locale: dateLocale })}`;
    }, [weekDays]);

    const handlePrevWeek = () => setCurrentWeek((d) => subWeeks(d, 1));
    const handleNextWeek = () => setCurrentWeek((d) => addWeeks(d, 1));
    const handleToday = () => setCurrentWeek(new Date());

    const formatDateStr = (d: Date) =>
      `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

    return (
      <div className={cn("bg-card rounded-xl border flex flex-col", className)}>
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b">
          <button
            onClick={handlePrevWeek}
            className="w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-accent transition-colors cursor-pointer"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <div className="text-center">
            <p className="text-sm font-semibold text-foreground">{weekRangeText}</p>
            {!isToday(currentWeek) && (
              <button
                onClick={handleToday}
                className="text-xs text-primary hover:underline cursor-pointer mt-0.5"
              >
                {t("components.schedule.week.today")}
              </button>
            )}
          </div>
          <button
            onClick={handleNextWeek}
            className="w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-accent transition-colors cursor-pointer"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>

        {/* Week Grid */}
        <ScrollArea className="flex-1">
          <div className="grid grid-cols-7 min-w-[600px]">
            {weekDays.map((day) => {
              const dayStr = formatDateStr(day);
              const dayItems = items.filter((item) => {
                // 检查日程的日期跨度是否包含当前天
                const start = item.span.dateStart;
                const end = item.span.dateEnd || start;
                if (!start) return false;
                return dayStr >= start && dayStr <= (end || start);
              });
              const isTodayDay = isToday(day);
              const isSelected = isSameDay(day, baseDate);

              return (
                <div
                  key={day.toISOString()}
                  className={cn(
                    "border-r last:border-r-0 flex flex-col min-h-52",
                    isSelected && "bg-primary/5",
                  )}
                >
                  {/* Day Header */}
                  <button
                    onClick={() => onSelectDate?.(day)}
                    className={cn(
                      "px-2 py-2 text-center cursor-pointer transition-colors hover:bg-accent/50",
                      isTodayDay && "bg-primary/5",
                    )}
                  >
                    <p className="text-xs text-muted-foreground uppercase">
                      {format(day, "EEE", { locale: dateLocale })}
                    </p>
                    <p
                      className={cn(
                        "text-sm font-semibold mt-0.5 w-7 h-7 mx-auto flex items-center justify-center rounded-full",
                        isTodayDay
                          ? "bg-primary text-primary-foreground"
                          : "text-foreground",
                      )}
                    >
                      {format(day, "d")}
                    </p>
                  </button>

                  {/* Items */}
                  <div className="flex-1 p-1.5 space-y-1.5">
                    {dayItems.length === 0 && (
                      <div className="h-8 flex items-center justify-center">
                        <span className="text-xs text-muted-foreground/50">—</span>
                      </div>
                    )}
                    {dayItems.map((item) => (
                      <button
                        key={item.id}
                        onClick={() => onItemClick?.(item)}
                        className={cn(
                          "w-full text-left px-2 py-1.5 rounded-md border text-xs transition-colors cursor-pointer",
                          getItemTypeColor(item.type),
                          item.done && "opacity-50 line-through",
                        )}
                      >
                        <span className="font-medium block truncate">{item.span.timeStart || "--:--"}</span>
                        <span className="block truncate opacity-90">{item.title}</span>
                      </button>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </div>
    );
  },
);

WeekView.displayName = "WeekView";
