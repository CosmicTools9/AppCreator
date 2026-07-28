/**
 * DateTimePicker · 日期时间选择器
 *
 * 日历 + 时间输入组合。
 */

import { format, parseISO } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";
import { Button } from "../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { Input } from "../ui/input";
import { CustomCalendar } from "./CustomCalendar";

export interface DateTimePickerProps {
  value: string | null | undefined;
  onChange: (value: string | null) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}

export function DateTimePicker({
  value,
  onChange,
  placeholder,
  disabled,
  className,
}: DateTimePickerProps) {
  const t = useT();
  const dtValue = value ? parseISO(value) : undefined;

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "w-full justify-start text-left font-normal",
            !value && "text-muted-foreground",
            className,
          )}
          disabled={disabled}
        >
          <CalendarIcon className="mr-2 h-4 w-4" />
          {value
            ? format(dtValue as Date, "yyyy/MM/dd HH:mm")
            : placeholder ?? t("autoform.pickDateTime", {}, { fallback: "选择日期时间" })}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0 border-0 bg-transparent shadow-xl">
        <div className="bg-[var(--popover)] rounded-md">
          <CustomCalendar
            selected={dtValue}
            onSelect={(date: Date | null) => {
              const timeStr = dtValue ? format(dtValue, "HH:mm") : "00:00";
              onChange(date ? `${format(date, "yyyy-MM-dd")}T${timeStr}` : null);
            }}
          />
          <div className="flex items-center gap-2 px-3 pb-3 border-t border-border pt-2">
            <Input
              type="time"
              className="flex-1 h-8"
              value={dtValue ? format(dtValue, "HH:mm") : "00:00"}
              onChange={(e) => {
                const dateStr = dtValue
                  ? format(dtValue, "yyyy-MM-dd")
                  : format(new Date(), "yyyy-MM-dd");
                onChange(`${dateStr}T${e.target.value}`);
              }}
            />
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}

DateTimePicker.displayName = "DateTimePicker";
