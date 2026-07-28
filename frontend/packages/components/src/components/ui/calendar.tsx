/**
 * Calendar Component
 *
 * A date picker component built on top of react-day-picker.
 */

import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { DayPicker } from "react-day-picker";

import { cn } from "../../lib/utils";

export type CalendarProps = React.ComponentProps<typeof DayPicker>;

function Calendar({
  className,
  classNames,
  showOutsideDays = true,
  ...props
}: CalendarProps) {
  return (
    <DayPicker
      showOutsideDays={showOutsideDays}
      weekStartsOn={0}
      className={cn("p-2", className)}
      classNames={{
        months: "flex flex-col sm:flex-row space-y-2 sm:space-x-2 sm:space-y-0",
        month: "space-y-2",
        caption: "flex justify-between items-center px-1 py-1",
        caption_label: "text-sm font-medium text-foreground",
        nav: "flex items-center gap-1",
        nav_button:
          "h-6 w-6 bg-transparent p-0 opacity-60 hover:opacity-100 flex items-center justify-center text-foreground transition-opacity",
        nav_button_previous: "",
        nav_button_next: "",
        table: "w-full border-collapse",
        head_row: "grid grid-cols-7 mb-1",
        head_cell:
          "text-muted-foreground w-8 h-8 font-normal text-xs flex items-center justify-center",
        row: "grid grid-cols-7 w-full gap-y-0.5",
        cell: "h-8 w-8 text-center text-sm p-0 relative flex items-center justify-center",
        day: "h-8 w-8 p-0 font-normal text-foreground rounded-sm flex items-center justify-center transition-colors hover:bg-accent",
        day_range_end: "day-range-end",
        day_selected:
          "bg-blue-500/30 text-white border border-blue-400 hover:bg-blue-500/40 hover:text-white focus:bg-blue-500/40 focus:text-white",
        day_today: "bg-accent text-accent-foreground font-medium",
        day_outside:
          "text-muted-foreground/40 opacity-60",
        day_disabled: "text-muted-foreground opacity-30 cursor-not-allowed",
        day_range_middle:
          "aria-selected:bg-accent aria-selected:text-accent-foreground",
        day_hidden: "invisible",
        ...classNames,
      } as any}
      components={{
          Chevron: (props: any) => {
            const Icon = props.orientation === "right" ? ChevronRight : ChevronLeft;
            return <Icon className={cn("h-4 w-4", props.className)} />;
          },
        } as any
      }
      {...props}
    />
  );
}
Calendar.displayName = "Calendar";

export { Calendar };
