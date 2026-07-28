/**
 * WorkspaceHub · 右侧工作区枢纽
 *
 * 核心职责：
 * 1. 读取 activeWorkspaceAtom，决定渲染哪个工作区
 * 2. 提供统一的 WorkspaceShell 外壳
 * 3. 管理面板开关的互斥性
 *
 * 使用示例：
 * ```tsx
 * <WorkspaceHub
 *   slots={[
 *     {
 *       id: "approval",
 *       title: <><ClipboardCheck className="w-5 h-5 text-primary" /> 审批工作区</>,
 *       content: <ApprovalPanel items={items} onApprove={...} />,
 *     },
 *     {
 *       id: "schedule",
 *       title: <><Calendar className="w-5 h-5 text-primary" /> 日程管理</>,
 *       content: <SchedulePanel />,
 *     },
 *   ]}
 * />
 * ```
 */

import * as React from "react";
import { useAtom } from "jotai";
import { activeWorkspaceAtom, closeWorkspaceAtom } from "./workspace-atoms";
import { WorkspaceShell } from "./WorkspaceShell";
import type { WorkspaceId } from "./workspace-atoms";

export interface WorkspaceSlot {
 /** 工作区标识 */
 id: WorkspaceId;
 /** Block ID — 用于关联 Block Capability */
 blockId?: string;
 /** 面板标题 */
 title: React.ReactNode;
 /** 面板内容 */
 content: React.ReactNode;
}

export interface WorkspaceHubProps {
 /** 工作区配置列表 */
 slots: WorkspaceSlot[];
 /** 自定义类名 */
 className?: string;
}

export const WorkspaceHub = React.forwardRef<
 HTMLDivElement,
 WorkspaceHubProps
>(({ slots, className }, ref) => {
 const [active] = useAtom(activeWorkspaceAtom);
 const [, close] = useAtom(closeWorkspaceAtom);

 const activeSlot = React.useMemo(
  () => slots.find((s) => s.id === active),
  [slots, active],
 );

 return (
  <div ref={ref} className={className}>
   <WorkspaceShell
    isOpen={!!activeSlot}
    onClose={close}
    title={activeSlot?.title ?? null}
   >
    {activeSlot ? (
     <React.Fragment key={activeSlot.id}>
      {activeSlot.content}
     </React.Fragment>
    ) : null}
   </WorkspaceShell>
  </div>
 );
});

WorkspaceHub.displayName = "WorkspaceHub";
