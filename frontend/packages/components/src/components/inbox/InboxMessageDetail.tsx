/**
 * InboxMessageDetail · 站内信消息详情
 *
 * 在右侧面板内展示单条消息的完整内容。
 * 包含发件人信息、标题、正文、回复输入框和操作按钮。
 */

import * as React from "react";
import {
  ArrowLeft,
  Trash2,
  Send,
  Mail,
  Clock,
} from "lucide-react";
import { cn } from "../../lib/utils";
import { Avatar, AvatarFallback } from "../ui/avatar";
import { Button } from "../ui/button";
import { Textarea } from "../ui/textarea";
import { Separator } from "../ui/separator";
import { useT } from "@alioth/i18n";
import type { InboxMessageDetailProps } from "./types";
import { getInboxTypeColor } from "./inbox-utils";

export const InboxMessageDetail = React.forwardRef<
  HTMLDivElement,
  InboxMessageDetailProps
>(({ message, onBack, onDelete, onReply, className }, ref) => {
  const t = useT();
  const [replyText, setReplyText] = React.useState("");

  const handleReply = () => {
    if (!replyText.trim()) return;
    onReply?.(message.id, replyText.trim());
    setReplyText("");
  };

  return (
    <div ref={ref} className={cn("flex flex-col h-full", className)}>
      {/* 头部：返回 + 操作 */}
      <div className="flex items-center justify-between px-6 py-3 border-b shrink-0">
        <Button
          variant="ghost"
          size="sm"
          onClick={onBack}
          className="gap-1 text-muted-foreground hover:text-foreground -gl-2"
        >
          <ArrowLeft className="w-4 h-4" />
          {t('components.inbox.backToList')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onDelete?.(message.id)}
          className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 gap-1"
        >
          <Trash2 className="w-4 h-4" />
          {t('common.delete')}
        </Button>
      </div>

      {/* 消息内容（可滚动） */}
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {/* 发件人信息 */}
        <div className="flex items-center gap-3 mb-5">
          <Avatar
            className={cn(
              "w-10 h-10 rounded-xl",
              getInboxTypeColor(message.type),
            )}
          >
            <AvatarFallback className="text-sm font-bold bg-transparent">
              {message.avatar || message.from.charAt(0)}
            </AvatarFallback>
          </Avatar>
          <div>
            <p className="font-semibold text-foreground">{message.from}</p>
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Clock className="w-3 h-3" />
              {message.time}
            </div>
          </div>
        </div>

        {/* 标题 */}
        <h3 className="text-base font-semibold text-foreground mb-3">
          {message.title}
        </h3>

        {/* 正文 */}
        <div className="text-sm text-muted-foreground leading-relaxed whitespace-pre-line">
          {message.content}
        </div>

        {/* 系统消息提示（如果是系统类型） */}
        {message.type === "system" && (
          <div className="mt-6 p-3 rounded-lg bg-muted/50 border text-xs text-muted-foreground flex items-start gap-2">
            <Mail className="w-4 h-4 shrink-0 mt-0.5" />
            <span>{t('components.inbox.systemMessageHint')}</span>
          </div>
        )}
      </div>

      {/* 回复区（非系统消息显示） */}
      {message.type !== "system" && (
        <>
          <Separator />
          <div className="px-6 py-4 shrink-0 space-y-3">
            <Textarea
              placeholder={t('components.inbox.replyPlaceholder')}
              value={replyText}
              onChange={(e) => setReplyText(e.target.value)}
              className="min-h-20 text-sm resize-none"
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault();
                  handleReply();
                }
              }}
            />
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">
                {t('components.inbox.quickSendHint')}
              </span>
              <Button
                size="sm"
                onClick={handleReply}
                disabled={!replyText.trim()}
                className="gap-1"
              >
                <Send className="w-3.5 h-3.5" />
                {t('components.inbox.reply')}
              </Button>
            </div>
          </div>
        </>
      )}
    </div>
  );
});

InboxMessageDetail.displayName = "InboxMessageDetail";
