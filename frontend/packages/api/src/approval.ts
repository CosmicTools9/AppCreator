import { createApiClient } from './client.js';
import type { ApiClientConfig } from './client.js';

// ── Shared types (union of both namespaces) ──────────────

export interface ApprovalFlowItem {
  id: number;
  name: string;
  code: string;
  status: string;
  category: string;
  'x-version'?: string;
  updated_at?: string;
  description?: string;
}

export interface ApprovalInstanceItem {
  id: number;
  flow_id: number;
  flow_name: string;
  applicant: string;
  status: string;
  timeline: TimelineNode[];
  created_at?: string;
}

export interface TimelineNode {
  node_name: string;
  status: 'active' | 'completed' | 'pending' | 'rejected';
  approver?: string;
  opinion?: string;
  updated_at?: string;
}
export type TimelineNodeStatus = TimelineNode['status'];

export interface MappedInstance extends ApprovalInstanceItem {
  [key: string]: unknown;
}

export interface EngineerItem {
  id: number;
  name: string;
}
export interface ApprovalRoleItem {
  id: number;
  name: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

// ── Factory ──────────────────────────────────────────────

export interface ApprovalApiConfig extends ApiClientConfig {
  apiBase: string;     // e.g., '/api/service/approval' or '/api/service/commitment'
  identityBase: string; // e.g., '/api/service/identity' or '/api/service/authority'
  identityEmployeePath: string; // '/engineers' or '/employees'
}

export function createApprovalApi(config: ApprovalApiConfig) {
  const client = createApiClient(config);

  return {
    // Flows
    getFlows: () => client.get('/approval-flows'),
    getFlow: (id: number) => client.get(`/approval-flows/${id}`),
    createFlow: (data: unknown) => client.post('/approval-flows', data),
    updateFlow: (id: number, data: unknown) => client.put(`/approval-flows/${id}`, data),
    publishFlow: (id: number) => client.post(`/approval-flows/${id}/publish`),

    // Instances
    getInstances: (params?: Record<string, string>) => client.get('/approval-instances', { params }),
    getEnrichedInstances: (params?: Record<string, string>) => client.get('/approval-instances/enriched', { params }),
    createInstance: (data: unknown) => client.post('/approval-instances', data),

    // Actions
    approve: (id: number, data: { opinion?: string }) => client.post(`/approval-instances/${id}/approve`, data),
    reject: (id: number, data: { opinion?: string }) => client.post(`/approval-instances/${id}/reject`, data),
    transfer: (id: number, data: { target_id: number; opinion?: string }) => client.post(`/approval-instances/${id}/transfer`, data),
    cc: (id: number, data: { target_id: number; opinion?: string }) => client.post(`/approval-instances/${id}/cc`, data),

    // Timeline
    getTimeline: (id: number) => client.get(`/approval-instances/${id}/timeline`),

    // Delegation rules
    getDelegationRules: () => client.get('/delegation-rules'),
    deleteDelegationRule: (id: number) => client.delete(`/delegation-rules/${id}`),
    createDelegationRule: (data: unknown) => client.post('/delegation-rules', data),

    // Identity services
    getEmployees: () => client.get(`${config.identityBase}${config.identityEmployeePath}`),
    getApprovalRoles: () => client.get(`${config.identityBase}/approval-roles`),
  };
}

export type ApprovalApi = ReturnType<typeof createApprovalApi>;
