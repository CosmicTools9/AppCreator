/**
 * MiniCalendar · 迷你月历
 *
 * 基于 react-day-picker v9 的轻量化月历组件。
 * 支持月份切换、日期选中、事件日期标记。
 */

import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { DayPicker } from "react-day-picker";

import { cn } from "../../lib/utils";
import { buttonVariants } from "../ui/button";
import type { MiniCalendarProps } from "./types";
import { useDateFnsLocale } from "./useDateFnsLocale";

export const MiniCalendar = React.forwardRef<HTMLDivElement, MiniCalendarProps>(
  ({ selectedDate, onSelectDate, eventDates = [], className }, ref) => {
    const [month, setMonth] = React.useState(selectedDate ?? new Date());

    // 同步外部 selectedDate 变化
    React.useEffect(() => {
      if (selectedDate) {
        setMonth(selectedDate);
      }
    }, [selectedDate]);

    const modifiers = React.useMemo(() => {
      const eventDateObjs = eventDates
        .map((d) => {
          const [y, m, day] = d.split("-").map(Number);
          return new Date(y, m - 1, day);
        })
        .filter((d) => !isNaN(d.getTime()));
      return { hasEvent: eventDateObjs };
    }, [eventDates]);

    const modifiersClassNames = {
      hasEvent:
        "relative after:absolute after:bottom-1 after:left-1/2 after:-translate-x-1/2 after:w-1 after:h-1 after:rounded-full after:bg-primary",
    };

    return (
      <div ref={ref} className={cn("bg-card rounded-xl border p-3", className)}>
        <DayPicker
          mode="single"
          selected={selectedDate}
          onSelect={(date) => date && onSelectDate?.(date)}
          month={month}
          onMonthChange={setMonth}
          locale={useDateFnsLocale()}
          showOutsideDays
          navLayout="around"
          modifiers={modifiers}
          modifiersClassNames={modifiersClassNames}
          className="w-full"
          classNames={{
            months: "flex flex-col",
            month: "grid grid-cols-[1fr_auto_1fr] gap-y-3 items-center",
            month_caption: "text-center text-sm font-medium",
            caption_label: "text-sm font-semibold",
            button_previous: cn(
              buttonVariants({ variant: "outline" }),
              "h-7 w-7 bg-transparent p-0 opacity-70 hover:opacity-100",
            ),
            button_next: cn(
              buttonVariants({ variant: "outline" }),
              "h-7 w-7 bg-transparent p-0 opacity-70 hover:opacity-100 gl-auto",
            ),
            chevron: "text-foreground",
            month_grid: "col-span-3 w-full border-collapse table-fixed",
            weekdays: "",
            weekday:
              "text-muted-foreground font-normal text-xs text-center h-8 p-0",
            weeks: "",
            week: "",
            day: "h-8 text-center text-sm p-0 relative",
            day_button: cn(
              buttonVariants({ variant: "ghost" }),
              "h-8 w-8 p-0 font-normal text-xs mx-auto rounded-md",
            ),
            selected:
              "bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground focus:bg-primary focus:text-primary-foreground rounded-md",
            today: "bg-accent text-accent-foreground rounded-md font-semibold",
            outside:
              "text-muted-foreground opacity-50 aria-selected:bg-accent/50 aria-selected:text-muted-foreground aria-selected:opacity-30",
            disabled: "text-muted-foreground opacity-50",
            hidden: "invisible",
          }}
          components={{
            Chevron: ({ orientation, className, ...props }) => {
              const Icon = orientation === "left" ? ChevronLeft : ChevronRight;
              return (
                <Icon
                  className={cn("h-4 w-4 text-foreground", className)}
                  {...props}
                />
              );
            },
          }}
        />
      </div>
    );
  },
);

MiniCalendar.displayName = "MiniCalendar";
