/**
 * Inbox Utils · 站内信共享工具函数
 */
import type { InboxMessageType } from "./types";

/** 根据消息类型返回对应的颜色类名 */
export function getInboxTypeColor(type: InboxMessageType): string {
  switch (type) {
    case "system":
      return "bg-muted text-muted-foreground";
    case "approval":
      return "bg-warning/10 dark:bg-warning/20 text-warning";
    case "mention":
      return "bg-info/10 dark:bg-info/20 text-info";
    case "message":
    default:
      return "bg-primary/10 dark:bg-primary/20 text-primary";
  }
}
