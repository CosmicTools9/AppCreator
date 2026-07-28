/**
 * QuickAddForm · 快速新建日程
 *
 * 可展开/收起的快速创建表单。
 * 支持标题、时间、地点输入，以及提醒设置（提前时间 + 提醒方式）。
 * 创建时写入 zc_id_plan，可附带创建关联 Event。
 */

import * as React from "react";
import { Plus, ChevronDown, ChevronUp, Bell } from "lucide-react";
import { cn } from "../../lib/utils";
import { Input } from "../ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { useT } from "@alioth/i18n";
import type { QuickAddFormProps, ReminderOffset } from "./types";

function useReminderOptions(): { value: ReminderOffset; label: string }[] {
  const t = useT();
  return [
    { value: 0, label: t("components.schedule.reminder.none") },
    { value: 5, label: t("components.schedule.reminder.5min") },
    { value: 15, label: t("components.schedule.reminder.15min") },
    { value: 30, label: t("components.schedule.reminder.30min") },
    { value: 60, label: t("components.schedule.reminder.1hour") },
    { value: 1440, label: t("components.schedule.reminder.1day") },
  ];
}

export const QuickAddForm = React.forwardRef<
  HTMLDivElement,
  QuickAddFormProps
>(({ onAdd, defaultDate, className }, ref) => {
  const t = useT();
  const reminderOptions = useReminderOptions();
  const [expanded, setExpanded] = React.useState(false);
  const [title, setTitle] = React.useState("");
  const [time, setTime] = React.useState("");
  const [location, setLocation] = React.useState("");
  const [reminderOffset, setReminderOffset] = React.useState<ReminderOffset>(15);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    const today = defaultDate || new Date().toISOString().split("T")[0];

    onAdd?.({
      title: title.trim(),
      duration: "1h",
      location: location || undefined,
      type: "meeting",
      span: {
        dateStart: today,
        dateEnd: today,
        timeStart: time || "09:00",
        timeEnd: undefined,
      },
      reminder:
        reminderOffset > 0
          ? { offset: reminderOffset, channel: "app" }
          : undefined,
    });

    // Reset
    setTitle("");
    setTime("");
    setLocation("");
    setReminderOffset(15);
    setExpanded(false);
  };

  return (
    <div
      ref={ref}
      className={cn("bg-card rounded-xl border overflow-hidden", className)}
    >
      <button
        type="button"
        onClick={() => setExpanded((e) => !e)}
        className="w-full flex items-center justify-between p-3 text-sm font-medium text-foreground hover:bg-accent/50 transition-colors cursor-pointer"
      >
        <span className="flex items-center gap-2">
          <Plus className="w-4 h-4 text-primary" />
          {t("components.schedule.quickAdd.title")}
        </span>
        {expanded ? (
          <ChevronUp className="w-4 h-4 text-muted-foreground" />
        ) : (
          <ChevronDown className="w-4 h-4 text-muted-foreground" />
        )}
      </button>

      {expanded && (
        <form
          onSubmit={handleSubmit}
          className="px-3 pb-3 space-y-3 border-t pt-3"
        >
          {/* Title */}
          <Input
            name="title"
            autoComplete="on"
            placeholder={t("components.schedule.quickAdd.eventTitlePlaceholder")}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="text-sm"
            autoFocus
          />

          {/* Time & Location */}
          <div className="flex gap-2">
            <Input
              name="time"
              autoComplete="on"
              type="time"
              placeholder={t("components.schedule.quickAdd.timePlaceholder")}
              value={time}
              onChange={(e) => setTime(e.target.value)}
              className="text-sm flex-1"
            />
            <Input
              name="location"
              autoComplete="on"
              placeholder={t("components.schedule.quickAdd.locationPlaceholder")}
              value={location}
              onChange={(e) => setLocation(e.target.value)}
              className="text-sm flex-1"
            />
          </div>

          {/* Reminder Setting */}
          <div className="flex items-center gap-2">
            <Bell className="w-4 h-4 text-muted-foreground shrink-0" />
            <Select
              value={String(reminderOffset)}
              onValueChange={(v) =>
                setReminderOffset(Number(v) as ReminderOffset)
              }
            >
              <SelectTrigger className="h-9 text-xs flex-1">
                <SelectValue placeholder={t("components.schedule.quickAdd.reminderPlaceholder")} />
              </SelectTrigger>
              <SelectContent>
                {reminderOptions.map((opt) => (
                  <SelectItem
                    key={opt.value}
                    value={String(opt.value)}
                    className="text-xs"
                  >
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Submit */}
          <button
            type="submit"
            disabled={!title.trim()}
            className={cn(
              "w-full py-2.5 text-sm font-medium rounded-lg transition-colors cursor-pointer",
              title.trim()
                ? "bg-primary text-primary-foreground hover:bg-primary/90"
                : "bg-muted text-muted-foreground cursor-not-allowed",
            )}
          >
            {t("components.schedule.quickAdd.submit")}
          </button>
        </form>
      )}
    </div>
  );
});

QuickAddForm.displayName = "QuickAddForm";
