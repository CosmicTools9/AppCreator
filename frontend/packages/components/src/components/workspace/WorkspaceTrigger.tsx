/**
 * WorkspaceTrigger · 顶栏工作区触发按钮
 *
 * 读取/设置 activeWorkspaceAtom，实现互斥切换。
 * 配合 WorkspaceHub 使用，不再各自管理 Sheet。
 *
 * 使用示例（放入 TopBar actions）：
 * ```tsx
 * <div className="flex items-center gap-3">
 *   <WorkspaceTrigger id="approval" icon={<ClipboardCheck />} pendingCount={3} />
 *   <WorkspaceTrigger id="schedule" icon={<Calendar />} />
 *   <WorkspaceTrigger id="inbox" icon={<Mail />} unreadCount={5} />
 *   <WorkspaceTrigger id="profile" icon={<UserCircle />} />
 * </div>
 * ```
 */

import * as React from "react";
import { useAtom } from "jotai";
import { cn } from "../../lib/utils";
import { activeWorkspaceAtom, toggleWorkspaceAtom } from "./workspace-atoms";
import type { WorkspaceId } from "./workspace-atoms";

export interface WorkspaceTriggerProps {
  /** 工作区标识 */
  id: WorkspaceId;
  /** 图标 */
  icon: React.ReactNode;
  /** 标题（用于 tooltip） */
  title?: string;
  /** 待处理数量徽标 */
  pendingCount?: number;
  /** 未读数量徽标 */
  unreadCount?: number;
  /** 自定义类名 */
  className?: string;
  /**
   * 可选点击回调。提供时替代默认 toggle 行为。
   * 可用于实现「只开不关」——组件只负责打开/切换，关闭由面板内部关闭按钮负责。
   */
  onClick?: () => void;
}

export const WorkspaceTrigger = React.forwardRef<
  HTMLButtonElement,
  WorkspaceTriggerProps
>(
  (
    { id, icon, title, pendingCount, unreadCount, className, onClick },
    ref,
  ) => {
    const [active] = useAtom(activeWorkspaceAtom);
    const [, toggle] = useAtom(toggleWorkspaceAtom);
    const isActive = active === id;

    const badgeCount = pendingCount ?? unreadCount;

    const handleClick = onClick ?? (() => toggle(id));

    return (
      <button
        ref={ref}
        onClick={handleClick}
        className={cn(
          "relative w-9 h-9 rounded-xl flex items-center justify-center",
          "transition-colors cursor-pointer bg-transparent",
          isActive
            ? "border border-foreground text-foreground"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
          className,
        )}
        title={title}
        aria-label={title}
        aria-pressed={isActive}
      >
        {icon}
        {typeof badgeCount === "number" && badgeCount > 0 && (
          <span
            className={cn(
              "absolute -top-0.5 -right-0.5 min-w-4 h-4 px-1 rounded-full text-xs font-bold flex items-center justify-center border-2",
              "bg-destructive text-destructive-foreground border-card",
            )}
          >
            {badgeCount > 99 ? "99+" : badgeCount}
          </span>
        )}
      </button>
    );
  },
);

WorkspaceTrigger.displayName = "WorkspaceTrigger";
