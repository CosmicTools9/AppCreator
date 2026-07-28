/**
 * Schedule Hooks · 日程数据获取
 */
import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@alioth/api";
import type { ScheduleItem, TodoItem, TodoObject, SchedulePlanType } from "@alioth/components";

// ── API 响应类型 ──

interface TodoApiItem {
  id: number;
  title: string;
  done: boolean;
  due_date?: string;
  status?: string;
  subject?: string;
  objects?: Array<{ id: number; name: string; object_type?: string }>;
}

interface ScheduleOverviewApiResponse {
  success: boolean;
  data: {
    today_event_count: number;
    pending_todo_count: number;
    upcoming_items: ApiScheduleItem[];
  };
}

interface ApiScheduleItem {
  id: number;
  title: string;
  type: string;
  span: {
    date_start?: string;
    date_end?: string;
    time_start?: string;
    time_end?: string;
  };
  duration: string;
  done: boolean;
  progress_pct: number;
}

// ── 类型校验 ──

const VALID_SCHEDULE_TYPES: SchedulePlanType[] = [
  "meeting", "sync", "client", "development", "dev",
  "team", "review", "personal", "other",
];

const VALID_OBJECT_TYPES = ["production", "bill", "other"] as const;

function isValidScheduleType(t: string): t is SchedulePlanType {
  return VALID_SCHEDULE_TYPES.includes(t as SchedulePlanType);
}

function toTodoObjects(items: NonNullable<TodoApiItem["objects"]>): TodoObject[] {
  return items.map((o) => {
    let objectType: TodoObject["type"] = undefined;
    if (o.object_type && VALID_OBJECT_TYPES.includes(o.object_type as typeof VALID_OBJECT_TYPES[number])) {
      objectType = o.object_type as TodoObject["type"];
    }
    return { id: o.id, name: o.name, type: objectType };
  });
}

// ── Hook ──

export function useScheduleOverview() {
  return useQuery({
    queryKey: ["schedule", "overview"],
    queryFn: async () => {
      const [overviewRes, todosRes] = await Promise.all([
        apiClient.get<ScheduleOverviewApiResponse>("/schedule/overview"),
        apiClient.get<{ success: boolean; data: TodoApiItem[] }>("/schedule/todos"),
      ]);
      const data = overviewRes?.data;
      const items: ScheduleItem[] = (data?.upcoming_items ?? []).map((item) => ({
        id: item.id,
        title: item.title,
        type: isValidScheduleType(item.type) ? item.type : ("other" as SchedulePlanType),
        span: {
          dateStart: item.span.date_start,
          dateEnd: item.span.date_end,
        },
        duration: item.duration,
        done: item.done,
        progressPct: item.progress_pct,
      }));
      const todos: TodoItem[] = (todosRes?.data ?? []).map((t) => ({
        id: t.id,
        title: t.title,
        done: t.done,
        dueDate: t.due_date,
        subject: t.subject,
        objects: toTodoObjects(t.objects ?? []),
        status: t.status,
      }));
      return {
        items,
        todos,
        todayEventCount: data?.today_event_count ?? 0,
        pendingTodoCount: todos.filter((t) => !t.done).length,
      };
    },
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}
