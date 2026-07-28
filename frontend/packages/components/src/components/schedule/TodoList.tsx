/**
 * TodoList · 待办清单
 *
 * 可勾选的待办事项列表，支持完成状态切换、进度统计。
 */

import * as React from "react";
import { CheckSquare, Package, FileText } from "lucide-react";
import { cn } from "../../lib/utils";
import { Checkbox } from "../ui/checkbox";
import type { TodoListProps, TodoObject } from "./types";
import { useT } from "@alioth/i18n";

function ObjectBadge({ obj }: { obj: TodoObject }) {
  const icon =
    obj.type === "production" ? (
      <Package className="w-3 h-3" />
    ) : obj.type === "bill" ? (
      <FileText className="w-3 h-3" />
    ) : null;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 text-xs px-1.5 py-0.5 rounded border",
        obj.type === "production" && "bg-success/10 dark:bg-success/20 text-success border-success/10",
        obj.type === "bill" && "bg-primary/10 dark:bg-primary/20 text-primary border-primary/10",
        (!obj.type || obj.type === "other") && "bg-muted text-muted-foreground border-muted",
      )}
    >
      {icon}
      {obj.name}
    </span>
  );
}

export const TodoList = React.forwardRef<HTMLDivElement, TodoListProps>(
  ({ items, onToggle, className }, ref) => {
    const t = useT();
    const completedCount = items.filter((t) => t.done).length;
    const totalCount = items.length;

    return (
      <div ref={ref} className={cn("space-y-2", className)}>
        {/* Header */}
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
            <CheckSquare className="w-4 h-4 text-success" />
            {t("components.todoList.title")}
          </h3>
          <span className="text-xs text-muted-foreground">
            {completedCount}/{totalCount}
          </span>
        </div>

        {/* List */}
        <div className="bg-card rounded-xl border p-2 space-y-0.5">
          {items.map((todo) => (
            <div
              key={todo.id}
              className={cn(
                "flex items-start gap-3 p-2.5 rounded-lg transition-colors",
                todo.done ? "opacity-60" : "hover:bg-accent",
              )}
            >
              <Checkbox
                id={`todo-${todo.id}`}
                checked={todo.done}
                onCheckedChange={() => onToggle?.(todo.id)}
                className="shrink-0 mt-0.5"
              />
              <div className="flex-1 min-w-0">
                <label
                  htmlFor={`todo-${todo.id}`}
                  className={cn(
                    "text-sm cursor-pointer block",
                    todo.done
                      ? "text-muted-foreground line-through"
                      : "text-foreground",
                  )}
                >
                  {todo.title}
                </label>
                {/* 客体列表（真正需要做的事） */}
                {todo.objects.length > 0 && (
                  <div className="flex flex-wrap gap-1 mt-1.5">
                    {todo.objects.map((obj) => (
                      <ObjectBadge key={obj.id} obj={obj} />
                    ))}
                  </div>
                )}
                {/* 主体 + 状态 */}
                {(todo.subject || todo.status) && (
                  <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
                    {todo.subject && <span>{t("components.todoList.assignee", { subject: todo.subject })}</span>}
                    {todo.status && (
                      <span
                        className={cn(
                          "px-1 py-0.5 rounded",
                          todo.done
                            ? "bg-success/10 dark:bg-success/20 text-success"
                            : "bg-warning/10 dark:bg-warning/20 text-warning",
                        )}
                      >
                        {todo.status}
                      </span>
                    )}
                  </div>
                )}
              </div>
            </div>
          ))}

          {items.length === 0 && (
            <div className="py-6 text-center text-sm text-muted-foreground">
              {t("components.todoList.empty")}
            </div>
          )}
        </div>
      </div>
    );
  },
);

TodoList.displayName = "TodoList";
