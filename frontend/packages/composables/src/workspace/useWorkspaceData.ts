/**
 * Workspace Data Hooks · 工作区数据获取
 *
 * 直接对接 Gateway 后端 API，供 WorkspaceDock ApprovalPanel / InboxPanel 使用。
 * 接口方案与 Module 级别的 CRUD API 一致（/global/overview 为聚合端点）。
 */
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@alioth/api';
import type { ApprovalItem } from '@alioth/components';
import type { InboxMessage } from '@alioth/components';
// ============================================
// API Response Types
// ============================================

interface GlobalOverviewApiResponse {
  success: boolean;
  data: {
    approvals: Array<{
      id: number;
      title: string;
      applicant: string;
      dept: string;
      code: string;
      status: string;
      time: string;
    }>;
    messages: Array<{
      id: number;
      from_user: string;
      title: string;
      content: string;
      time: string;
      unread: boolean;
      msg_type: string;
    }>;
  };
}

// ============================================
// Normalization Helpers
// ============================================

function normalizeApprovalStatus(status: string): ApprovalItem['status'] {
  const lower = status.toLowerCase();
  if (lower === 'approved' || lower === 'rejected') return lower;
  return 'pending';
}

function normalizeMessageType(type: string): InboxMessage['type'] {
  const lower = type.toLowerCase();
  if (lower === 'system') return 'system';
  if (lower === 'approval') return 'approval';
  return 'message';
}

// ============================================
// Workspace Overview Hook
// ============================================

export interface WorkspaceOverviewData {
  approvals: ApprovalItem[];
  messages: InboxMessage[];
}

/** 获取全局工作区概览（审批 + 站内信） */
export function useWorkspaceOverview() {
  return useQuery<WorkspaceOverviewData>({
    queryKey: ['workspace', 'global-overview'],
    queryFn: async () => {
      const res = await apiClient.get<GlobalOverviewApiResponse>('/global/overview');
      const data = res?.data;
      return {
        approvals: (data?.approvals ?? []).map((item) => ({
          id: item.id,
          title: item.title,
          applicant: item.applicant,
          dept: item.dept,
          code: item.code,
          status: normalizeApprovalStatus(item.status),
          time: item.time,
        })),
        messages: (data?.messages ?? []).map((item) => ({
          id: item.id,
          from: item.from_user,
          title: item.title,
          content: item.content,
          time: item.time,
          unread: item.unread,
          type: normalizeMessageType(item.msg_type),
        })),
      };
    },
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}
