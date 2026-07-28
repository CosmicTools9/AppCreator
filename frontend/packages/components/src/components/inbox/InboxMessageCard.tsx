/**
 * InboxMessageCard · 站内信消息卡片
 *
 * 用于消息列表中单条消息的预览展示。
 * 显示发件人头像、标题、摘要、时间和未读状态。
 */

import * as React from "react";
import { Mail, AtSign, Trash2 } from "lucide-react";
import { cn } from "../../lib/utils";
import { Avatar, AvatarFallback } from "../ui/avatar";
import type { InboxMessageCardProps } from "./types";
import { useT } from "@alioth/i18n";
import { getInboxTypeColor } from "./inbox-utils";

export const InboxMessageCard = React.forwardRef<
  HTMLDivElement,
  InboxMessageCardProps
>(({ message, selected, onClick, onDelete, className }, ref) => {
  const t = useT();
  return (
    <div
      ref={ref}
      onClick={() => onClick?.(message)}
      className={cn(
        "group relative p-4 rounded-xl border cursor-pointer transition-colors transition-shadow",
        selected
          ? "border-primary bg-primary/5"
          : "bg-card hover:border-muted-foreground/30",
        className,
      )}
    >
      <div className="flex items-start gap-3">
        {/* 头像 */}
        <Avatar
          className={cn(
            "w-9 h-9 rounded-lg shrink-0",
            getInboxTypeColor(message.type),
          )}
        >
          <AvatarFallback className="text-xs font-bold bg-transparent">
            {message.avatar || message.from.charAt(0)}
          </AvatarFallback>
        </Avatar>

        {/* 内容 */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <p
              className={cn(
                "text-sm truncate",
                message.unread
                  ? "font-semibold text-foreground"
                  : "font-medium text-foreground",
              )}
            >
              {message.title}
            </p>
            {message.unread && (
              <span className="w-1.5 h-1.5 rounded-full bg-destructive shrink-0" />
            )}
          </div>

          <p className="text-xs text-muted-foreground mt-0.5 line-clamp-1">
            {message.content}
          </p>

          <div className="flex items-center justify-between mt-2">
            <div className="flex items-center gap-1.5">
              {message.type === "system" && <Mail className="w-3.5 h-3.5" />}
              {message.type === "mention" && <AtSign className="w-3.5 h-3.5" />}
              <span className="text-xs text-muted-foreground">
                {message.time}
              </span>
            </div>

            {/* 删除按钮（hover 显示） */}
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDelete?.(message.id);
              }}
              className="opacity-0 group-hover:opacity-100 p-1 rounded-md text-muted-foreground/50 hover:text-destructive hover:bg-destructive/10 transition-opacity"
              title={t("common.delete")}
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
});

InboxMessageCard.displayName = "InboxMessageCard";
