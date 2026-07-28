//! 日程页面工厂
//!
//! 消除各模块 SchedulePage 的复制粘贴。

import * as React from "react";
import { useNavigate } from "react-router";
import { CalendarDays, ArrowLeft } from "lucide-react";
import { SchedulePanel } from "./SchedulePanel";
import { useT } from "@alioth/i18n";
import type { TranslateFunction } from "@alioth/i18n";
import type { ScheduleItem, TodoItem } from "./types";

export interface SchedulePageOptions {
  /** 模块标识（如 "clients"、"vendors"），用于 i18n key */
  moduleName: string;
  /** 页面标题 i18n key */
  titleKey?: string;
  /** 页面副标题 i18n key */
  subtitleKey?: string;
  /** 面板标题 i18n key */
  panelTitleKey?: string;
  /** 标题 fallback */
  titleFallback?: string;
  /** 副标题 fallback */
  subtitleFallback?: string;
  /** 面板标题 fallback */
  panelTitleFallback?: string;
  /** 初始日程数据工厂（接收 t 函数以支持翻译和动态日期） */
  getScheduleItems: (t: TranslateFunction) => ScheduleItem[];
  /** 初始待办数据工厂 */
  getTodos?: (t: TranslateFunction) => TodoItem[];
}

/**
 * 创建模块日程页面
 *
 * @example
 * ```tsx
 * // pages/SchedulePage.tsx
 * import { createSchedulePage } from "@alioth/components/schedule";
 *
 * export default createSchedulePage({
 *   moduleName: "clients",
 *   titleFallback: "客户拜访日程",
 *   getScheduleItems: () => [...],
 *   getTodos: () => [...],
 * });
 * ```
 */
export function createSchedulePage(options: SchedulePageOptions) {
  const {
    moduleName,
    titleKey = `${moduleName}.schedule.title`,
    subtitleKey = `${moduleName}.schedule.subtitle`,
    panelTitleKey = `${moduleName}.schedule.panelTitle`,
    titleFallback = "Schedule Management",
    subtitleFallback = "Manage schedules and tasks",
    panelTitleFallback = "Schedule Panel",
    getScheduleItems,
    getTodos,
  } = options;

  return function SchedulePage(): React.ReactElement {
    const t = useT();
    const navigate = useNavigate();
    const [viewMode, setViewMode] = React.useState<"month" | "week" | "list">("week");
    const [selectedDate, setSelectedDate] = React.useState<Date>(new Date());

    const scheduleItems = React.useMemo<ScheduleItem[]>(() => getScheduleItems(t), [t]);
    const todos = React.useMemo<TodoItem[]>(() => (getTodos ? getTodos(t) : []), [t]);

    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <button
              onClick={() => navigate("/")}
              className="w-9 h-9 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-accent transition-colors cursor-pointer"
            >
              <ArrowLeft className="w-4 h-4" />
            </button>
            <div>
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                {t(titleKey, {}, { fallback: titleFallback })}
              </h1>
              <p className="text-sm text-muted-foreground mt-1">
                {t(subtitleKey, {}, { fallback: subtitleFallback })}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-card rounded-xl border">
          <div className="flex items-center gap-2 px-4 py-3 border-b">
            <CalendarDays className="w-4 h-4 text-primary" />
            <h3 className="text-sm font-semibold text-foreground">
              {t(panelTitleKey, {}, { fallback: panelTitleFallback })}
            </h3>
          </div>
          <div className="p-4">
            <SchedulePanel
              items={scheduleItems}
              todos={todos}
              viewMode={viewMode}
              onViewModeChange={setViewMode}
              selectedDate={selectedDate}
              onSelectDate={setSelectedDate}
              onItemClick={undefined}
              onToggleTodo={undefined}
            />
          </div>
        </div>
      </div>
    );
  };
}
