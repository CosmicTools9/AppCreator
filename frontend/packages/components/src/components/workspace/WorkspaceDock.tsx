/**
 * WorkspaceDock · 右侧工作区 Docked 模式
 *
 * 与 WorkspaceHub（Sheet 覆盖模式）互斥的另一种形态。
 * Docked 模式下主内容区收缩为 w-3/4，右侧面板以 w-1/4 挤压进入。
 *
 * 使用示例：
 * ```tsx
 * const [active] = useAtom(activeWorkspaceAtom);
 * const isOpen = !!active;
 *
 * <div className={cn("flex flex-col overflow-hidden", isOpen ? "w-3/4" : "flex-1")}>
 *   <TopBar ... />
 *   <ContentArea>...</ContentArea>
 * </div>
 *
 * {isOpen && <WorkspaceDock slots={[...]} />}
 * ```
 */

import * as React from "react";
import { useAtom } from "jotai";
import { cn } from "../../lib/utils";
import { activeWorkspaceAtom } from "./workspace-atoms";
import type { WorkspaceSlot } from "./WorkspaceHub";

export interface WorkspaceDockProps {
  /** 工作区配置列表 */
  slots: WorkspaceSlot[];
  /** 自定义类名 */
  className?: string;
}

export const WorkspaceDock = React.forwardRef<
  HTMLDivElement,
  WorkspaceDockProps
>(({ slots, className }, ref) => {
  const [active] = useAtom(activeWorkspaceAtom);

  const activeSlot = React.useMemo(
    () => slots.find((s) => s.id === active),
    [slots, active],
  );

  if (!activeSlot) return null;

  return (
    <div
      ref={ref}
      data-right-sidebar
      className={cn("h-full w-full flex flex-col", className)}
    >
      {activeSlot.content}
    </div>
  );
});

WorkspaceDock.displayName = "WorkspaceDock";
