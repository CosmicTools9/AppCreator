/**
 * DateRangePicker · 日期范围选择器
 *
 * 两个 Popover 日历，分别选择开始和结束日期。
 */

import { parseISO, format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";
import { Button } from "../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { CustomCalendar } from "./CustomCalendar";

export interface DateRangeValue {
  start: string | null;
  end: string | null;
}

export interface DateRangePickerProps {
  value: DateRangeValue;
  onChange: (value: DateRangeValue) => void;
  disabled?: boolean;
  className?: string;
}

export function DateRangePicker({
  value,
  onChange,
  disabled,
  className,
}: DateRangePickerProps) {
  const t = useT();
  const startVal = value.start ? parseISO(value.start) : undefined;
  const endVal = value.end ? parseISO(value.end) : undefined;

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className={cn(
              "flex-1 justify-start text-left font-normal",
              !value.start && "text-muted-foreground",
            )}
            disabled={disabled}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {value.start
              ? format(startVal as Date, "yyyy/MM/dd")
              : t("autoform.startDate", {}, { fallback: "开始日期" })}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0 border-0 bg-transparent shadow-xl">
          <CustomCalendar
            selected={startVal}
            onSelect={(d) =>
              onChange({
                ...value,
                start: d ? format(d, "yyyy-MM-dd") : null,
              })
            }
          />
        </PopoverContent>
      </Popover>
      <span className="text-muted-foreground">—</span>
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className={cn(
              "flex-1 justify-start text-left font-normal",
              !value.end && "text-muted-foreground",
            )}
            disabled={disabled}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {value.end
              ? format(endVal as Date, "yyyy/MM/dd")
              : t("autoform.endDate", {}, { fallback: "结束日期" })}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0 border-0 bg-transparent shadow-xl">
          <CustomCalendar
            selected={endVal}
            onSelect={(d) =>
              onChange({
                ...value,
                end: d ? format(d, "yyyy-MM-dd") : null,
              })
            }
          />
        </PopoverContent>
      </Popover>
    </div>
  );
}

DateRangePicker.displayName = "DateRangePicker";
