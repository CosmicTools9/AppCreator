/**
 * Identity Service Hooks · 人员角色数据获取
 *
 * 提供 Framework 级别的 typed hooks，不包含 UI 组件。
 * 各 NS 的 Module 前端自行实现 UI，调用这些 hooks 获取数据。
 *
 * 端点：Framework authority crate 统一挂载于 `/service/authority/*`
 * （Alioth/AVIC-CAASEC 一致）；列表响应为 `{ list, items, total, ... }`，
 * 详情响应为实体本体。
 */
import { useQuery } from '@tanstack/react-query';
import { apiClient } from '@alioth/api';
import type { Engineer, SkillTag, ApprovalRole, CCBMember } from './types.js';

type ListEnvelope<T> = { items?: T[]; list?: T[] };
const listOf = <T>(r: ListEnvelope<T>): T[] => r.items ?? r.list ?? [];

// ── Engineer（后端资源名 employees） ──────────────────────────

export function useEngineerList() {
  return useQuery<Engineer[]>({
    queryKey: ['identity', 'engineers'],
    queryFn: () =>
      apiClient
        .get<ListEnvelope<Engineer>>('/service/authority/employees')
        .then(listOf),
  });
}

export function useEngineerDetail(id: number | null) {
  return useQuery<Engineer | null>({
    queryKey: ['identity', 'engineers', id],
    queryFn: () =>
      id != null
        ? apiClient.get<Engineer>(`/service/authority/employees/${id}`)
        : null,
    enabled: id != null,
  });
}

// ── SkillTag ──────────────────────────────────────────────────

export function useSkillTagList() {
  return useQuery<SkillTag[]>({
    queryKey: ['identity', 'skill-tags'],
    queryFn: () =>
      apiClient
        .get<ListEnvelope<SkillTag>>('/service/authority/skill-tags')
        .then(listOf),
  });
}

// ── ApprovalRole ──────────────────────────────────────────────

export function useApprovalRoleList() {
  return useQuery<ApprovalRole[]>({
    queryKey: ['identity', 'approval-roles'],
    queryFn: () =>
      apiClient
        .get<ListEnvelope<ApprovalRole>>('/service/authority/approval-roles')
        .then(listOf),
  });
}

// ── CCBMember（后端资源名 approvers） ─────────────────────────

export function useCCBMemberList() {
  return useQuery<CCBMember[]>({
    queryKey: ['identity', 'ccb-members'],
    queryFn: () =>
      apiClient
        .get<ListEnvelope<CCBMember>>('/service/authority/approvers')
        .then(listOf),
  });
}
