/**
 * CustomCalendar · 日期选择弹层面板
 *
 * 与 AutoForm 内联日历共享同一实现，避免重复。
 */

import { useState } from "react";
import {
  format,
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  addDays,
  isSameMonth,
  isSameDay,
  addMonths,
  subMonths,
} from "date-fns";
import { ChevronDown } from "lucide-react";
import { cn } from "../../lib/utils";

export interface CustomCalendarProps {
  selected?: Date;
  onSelect: (date: Date | null) => void;
}

export function CustomCalendar({ selected, onSelect }: CustomCalendarProps) {
  const [currentMonth, setCurrentMonth] = useState(selected || new Date());

  const monthStart = startOfMonth(currentMonth);
  const monthEnd = endOfMonth(monthStart);
  const calendarStart = startOfWeek(monthStart, { weekStartsOn: 0 });
  const calendarEnd = endOfWeek(monthEnd, { weekStartsOn: 0 });

  const days: Date[] = [];
  let day = calendarStart;
  while (day <= calendarEnd) {
    days.push(day);
    day = addDays(day, 1);
  }

  const weekDays = ["日", "一", "二", "三", "四", "五", "六"];

  return (
    <div className="bg-[#1f1f1f] rounded-md p-3 w-[272px]">
      <div className="flex items-center justify-between mb-2">
        <div className="text-white text-sm font-medium flex items-center">
          {format(currentMonth, "yyyy年MM月")}
          <ChevronDown className="h-3 w-3 gl-1 opacity-50" />
        </div>
        <div className="flex items-center gap-0.5">
          <button
            type="button"
            className="h-6 w-6 flex items-center justify-center text-white/50 hover:text-white text-xs"
            onClick={() => setCurrentMonth(subMonths(currentMonth, 1))}
          >
            ↑
          </button>
          <button
            type="button"
            className="h-6 w-6 flex items-center justify-center text-white/50 hover:text-white text-xs"
            onClick={() => setCurrentMonth(addMonths(currentMonth, 1))}
          >
            ↓
          </button>
        </div>
      </div>

      <div className="grid grid-cols-7 mb-1">
        {weekDays.map((d) => (
          <div
            key={d}
            className="h-7 flex items-center justify-center text-xs text-white/40"
          >
            {d}
          </div>
        ))}
      </div>

      <div className="grid grid-cols-7 gap-y-0.5">
        {days.map((d, i) => {
          const isCurrentMonth = isSameMonth(d, currentMonth);
          const isSelected = selected && isSameDay(d, selected);
          const isToday = isSameDay(d, new Date());

          return (
            <button
              key={i}
              type="button"
              className={cn(
                "h-8 w-8 flex items-center justify-center text-sm rounded transition-colors mx-auto",
                !isCurrentMonth && "text-white/25",
                isCurrentMonth && !isSelected && "text-white/85 hover:bg-white/10",
                isSelected && "bg-[#1677ff] text-white",
                isToday && !isSelected && "text-[#1677ff] border border-[#1677ff]/40",
              )}
              onClick={() => onSelect(d)}
            >
              {format(d, "d")}
            </button>
          );
        })}
      </div>

      <div className="flex justify-between items-center mt-2 pt-2 border-t border-white/10">
        <button
          type="button"
          className="text-xs text-[#1677ff] hover:opacity-80 px-1 py-0.5"
          onClick={() => onSelect(null)}
        >
          清除
        </button>
        <button
          type="button"
          className="text-xs text-[#1677ff] hover:opacity-80 px-1 py-0.5"
          onClick={() => onSelect(new Date())}
        >
          今天
        </button>
      </div>
    </div>
  );
}

CustomCalendar.displayName = "CustomCalendar";
