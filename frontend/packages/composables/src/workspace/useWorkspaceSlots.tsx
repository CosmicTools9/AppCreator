/**
 * Workspace Slots · 模块 + 系统级工作区面板工厂
 *
 * 支持两种模式：
 * 1. 系统级能力（简单 capId）— 使用内置 Panel 组件
 * 2. 模块级能力（命名空间 ID "moduleId/capId"）— 从 Block Capability 加载
 *
 * Panel 组件直接对接后端 API（schedule/overview + global/overview），
 * 调用方仅需控制各 Slot 的显示与回调绑定。
 */
import { useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@alioth/api";
import * as React from "react";
import { useT } from "@alioth/i18n";
import { useAtom } from "jotai";
import { Bot, ClipboardCheck, Calendar, Mail, UserCircle, X } from "lucide-react";
import { closeWorkspaceAtom } from "@alioth/components";
import type { WorkspaceSlot } from "@alioth/components";
import { AIWorkspace } from "@alioth/components";
import { ApprovalPanel } from "@alioth/components";
import { SchedulePanel } from "@alioth/components";
import { InboxPanel } from "@alioth/components";
import { useScheduleOverview } from "../schedule/hooks";
import { useWorkspaceOverview } from "./useWorkspaceData";
import type { AgentOption } from "@alioth/components";
import type { InboxMessage, InboxSendParams } from "@alioth/components";
import type { ContactOption } from "@alioth/components";
export interface WorkspaceSlotConfig {
 /** 当前模块 ID — 用于命名空间化 IDs */
 moduleId?: string;
 /** AI 助手（undefined 则不渲染 AI slot） */
 ai?: {
  onSend: (message: string, pageContext?: unknown) => Promise<string>;
  agents?: AgentOption[];
  agentCode?: string;
  onAgentChange?: (code: string) => void;
 };
 /** 审批工作区（undefined 则不渲染 approval slot） */
 approval?: {
  /** Block ID — 省略时使用默认审批 block */
  blockId?: string;
  onApprove?: (id: string | number) => void;
  onReject?: (id: string | number) => void;
 };
 /** 日程管理（undefined 则不渲染 schedule slot） */
 schedule?: Record<string, never>;
 /** 站内信（undefined 则不渲染 inbox slot） */
 inbox?: {
  onMessageClick?: (message: InboxMessage) => void;
  onDelete?: (id: string | number) => void;
  onMarkAllRead?: () => void;
  onReply?: (id: string | number, content: string) => void;
  onSend?: (params: InboxSendParams) => void;
  contacts?: ContactOption[];
 };
 /** 用户档案（undefined 则不渲染 profile slot） */
 profile?: {
  content?: React.ReactNode;
 };
}

export interface WorkspaceSlotsResult {
 /** WorkspaceDock 使用的 slots */
 slots: WorkspaceSlot[];
 /** 待审批数量（徽标用） */
 pendingCount: number;
 /** 未读消息数量（徽标用） */
 unreadCount: number;
 /** 日程事件数量（徽标用） */
 scheduleEventCount: number;
}

/** 统一的工作区面板头部 */
function WorkspaceHeader({
 icon,
 title,
 badge,
}: {
 icon: React.ReactNode;
 title: string;
 badge?: React.ReactNode;
}) {
 const [, close] = useAtom(closeWorkspaceAtom);
 const t = useT();
 return (
  <div className="flex items-center justify-between border-b h-16 px-4 shrink-0 w-full">
   <div className="flex items-center gap-2 min-w-0">
    {icon}
    <span className="font-semibold text-sm truncate">{title}</span>
    {badge}
   </div>
   <button
    onClick={close}
    className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted transition-colors"
    aria-label={t("common.close")}
   >
    <X className="h-4 w-4" />
   </button>
  </div>
 );
}

/** 构建命名空间化的 slot ID */
function nsId(moduleId: string | undefined, capId: string): string {
 return moduleId ? `${moduleId}/${capId}` : capId;
}

/**
 * 加载 Block Capability 的面板内容
 * 若 block 已注册（通过 window.aliothBlockComponents），
 * 使用其 render() 方法渲染；否则渲染内置 Panel 内容
 */
function BlockSlotContent({
 blockId,
 fallback,
}: {
 blockId: string;
 fallback: React.ReactNode;
}): React.ReactElement {
 const blockComp = React.useMemo(() => {
  try {
   const reg = (window as any).aliothBlockComponents as Record<string, any>;
   return reg?.[blockId] ?? null;
  } catch { return null; }
 }, [blockId]);

 if (blockComp?.render) {
  return blockComp.render({ compact: true });
 }
 return <>{fallback}</>;
}


export function useWorkspaceSlots(config: WorkspaceSlotConfig): WorkspaceSlotsResult {
 const queryClient = useQueryClient();

 const t = useT();
 const { moduleId } = config;

 // ============================================
 // 数据获取 — Panel 直接对接后端 API
 // ============================================

 const scheduleQuery = useScheduleOverview();
 const workspaceQuery = useWorkspaceOverview();

 const scheduleItems = scheduleQuery.data?.items ?? [];
 const scheduleLoading = scheduleQuery.isLoading;

 const approvals = workspaceQuery.data?.approvals ?? [];
 const inboxMessages = workspaceQuery.data?.messages ?? [];
 const workspaceLoading = workspaceQuery.isLoading;

 const pendingCount = approvals.filter((i) => i.status === "pending").length;
 const unreadCount = inboxMessages.filter((m) => m.unread).length;
 const scheduleEventCount = scheduleItems.length;

 const slots = React.useMemo(() => {
  const result: WorkspaceSlot[] = [];

  if (config.ai) {
   result.push({
    id: nsId(moduleId, "ai"),
    blockId: "workspace-ai",
    title: (
     <span className="flex items-center gap-2">
      <Bot className="w-5 h-5 text-primary" />
      {t("moduleLayout.aiAssistant")}
     </span>
    ),
    content: (
     <div className="flex flex-col h-full w-full">
      <WorkspaceHeader
       icon={<Bot className="w-5 h-5 text-primary" />}
       title={t("moduleLayout.aiAssistant")}
      />
      <div className="flex-1 overflow-hidden flex flex-col w-full">
       <AIWorkspace
        onSend={config.ai.onSend}
        agents={config.ai.agents}
        agentCode={config.ai.agentCode}
        onAgentChange={config.ai.onAgentChange}
       />
      </div>
     </div>
    ),
   });
  }

  if (config.approval) {
   const blockId = config.approval.blockId ?? "block-approval-execution";
   // 审批操作 — API mutation，完成后自动刷新面板数据
   const onApprove = config.approval.onApprove ?? (async (id: string | number) => {
    await apiClient.post(`/approvals/${id}/approve`);
    queryClient.invalidateQueries({ queryKey: ["workspace", "global-overview"] });
   });
   const onReject = config.approval.onReject ?? (async (id: string | number) => {
    await apiClient.post(`/approvals/${id}/reject`);
    queryClient.invalidateQueries({ queryKey: ["workspace", "global-overview"] });
   });
   result.push({
    id: nsId(moduleId, "approval"),
    blockId,
    title: (
     <span className="flex items-center gap-2">
      <ClipboardCheck className="w-5 h-5 text-primary" />
      {t("moduleLayout.approvalWorkspace")}
     </span>
    ),
    content: (
     <div className="flex flex-col h-full w-full">
      <WorkspaceHeader
       icon={<ClipboardCheck className="w-5 h-5 text-primary" />}
       title={t("moduleLayout.approvalWorkspace")}
       badge={
        pendingCount > 0 ? (
         <span className="text-xs px-2 py-0.5 rounded-full bg-destructive/10 dark:bg-destructive/20 text-destructive font-medium">
          {pendingCount} {t("moduleLayout.pending")}
         </span>
        ) : undefined
       }
      />
      <div className="flex-1 overflow-hidden flex flex-col w-full">
       <BlockSlotContent
        blockId={blockId}
        fallback={
         <ApprovalPanel
          items={approvals}
          loading={workspaceLoading}
          onApprove={onApprove}
          onReject={onReject}
         />
        }
       />
      </div>
     </div>
    ),
   });
  }
  if (config.schedule) {
   result.push({
    id: nsId(moduleId, "schedule"),
    blockId: "workspace-schedule",
    title: (
     <span className="flex items-center gap-2">
      <Calendar className="w-5 h-5 text-primary" />
      {t("moduleLayout.scheduleManagement")}
     </span>
    ),
    content: (
     <div className="flex flex-col h-full w-full">
      <WorkspaceHeader
       icon={<Calendar className="w-5 h-5 text-primary" />}
       title={t("moduleLayout.scheduleManagement")}
      />
      <div className="flex-1 overflow-hidden flex flex-col w-full">
       <SchedulePanel
        items={scheduleItems}
        todos={scheduleQuery.data?.todos ?? []}
        loading={scheduleLoading}
       />
      </div>
     </div>
    ),
   });
  }

  if (config.inbox) {
   // 站内信操作 — API mutation，完成后自动刷新面板数据
   const onMessageClick = config.inbox.onMessageClick ?? (async (message: InboxMessage) => {
    await apiClient.patch(`/messages/${message.id}/read`);
    queryClient.invalidateQueries({ queryKey: ["workspace", "global-overview"] });
   });
   const onDelete = config.inbox.onDelete ?? (async (id: string | number) => {
    await apiClient.delete(`/messages/${id}`);
    queryClient.invalidateQueries({ queryKey: ["workspace", "global-overview"] });
   });
   const onMarkAllRead = config.inbox.onMarkAllRead ?? (async () => {
    const unreadIds = inboxMessages.filter((m) => m.unread).map((m) => m.id);
    if (unreadIds.length === 0) return;
    await Promise.all(unreadIds.map((id) => apiClient.patch(`/messages/${id}/read`)));
    queryClient.invalidateQueries({ queryKey: ["workspace", "global-overview"] });
   });
   result.push({
    id: nsId(moduleId, "inbox"),
    blockId: "workspace-inbox",
    title: (
     <span className="flex items-center gap-2">
      <Mail className="w-5 h-5 text-primary" />
      {t("moduleLayout.inbox")}
     </span>
    ),
    content: (
     <div className="flex flex-col h-full w-full">
      <WorkspaceHeader
       icon={<Mail className="w-5 h-5 text-primary" />}
       title={t("moduleLayout.inbox")}
       badge={
        unreadCount > 0 ? (
         <span className="text-xs px-2 py-0.5 rounded-full bg-destructive/10 dark:bg-destructive/20 text-destructive font-medium">
          {unreadCount} {t("moduleLayout.unread")}
         </span>
        ) : undefined
       }
      />
      <div className="flex-1 overflow-hidden flex flex-col w-full">
       <InboxPanel
        messages={inboxMessages}
        loading={workspaceLoading}
        onMessageClick={onMessageClick}
        onDelete={onDelete}
        onMarkAllRead={onMarkAllRead}
        onReply={config.inbox.onReply}
        onSend={config.inbox.onSend}
        contacts={config.inbox.contacts}
       />
      </div>
     </div>
    ),
   });
  }
  if (config.profile) {
   result.push({
    id: nsId(moduleId, "profile"),
    blockId: "workspace-profile",
    title: (
     <span className="flex items-center gap-2">
      <UserCircle className="w-5 h-5 text-primary" />
      {t("moduleLayout.userProfile")}
     </span>
    ),
    content: (
     <div className="flex flex-col h-full w-full">
      <WorkspaceHeader
       icon={<UserCircle className="w-5 h-5 text-primary" />}
       title={t("moduleLayout.userProfile")}
      />
      <div className="flex-1 overflow-auto flex flex-col w-full">
       {config.profile.content ?? (
        <div className="p-6 w-full">
         <div className="bg-muted/30 rounded-lg border border-border p-4 space-y-2">
          <p className="font-semibold text-foreground text-sm">{t("moduleLayout.currentUser")}</p>
          <p className="text-sm text-muted-foreground">{t("moduleLayout.username")}: {t("moduleLayout.demoUser")}</p>
          <p className="text-sm text-muted-foreground">{t("moduleLayout.department")}: {t("moduleLayout.itDepartment")}</p>
          <p className="text-xs text-muted-foreground pt-2">{t("moduleLayout.profilePlaceholder")}</p>
         </div>
        </div>
       )}
      </div>
     </div>
    ),
   });
  }

  return result;
 }, [
  config,
  moduleId,
  approvals,
  inboxMessages,
  scheduleItems,
  pendingCount,
  unreadCount,
  scheduleLoading,
  workspaceLoading,
  t,
 ]);

 return { slots, pendingCount, unreadCount, scheduleEventCount };
}

export { nsId };
