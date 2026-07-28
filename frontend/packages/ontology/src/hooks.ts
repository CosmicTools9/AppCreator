//! Alioth Ontology 前端业务 Hooks
//!
//! 提供基于本体坐标的实体查询、状态转移、生命周期操作等能力。

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@alioth/api";
import type {
  OntologyExpression,
  OntologyCoordinate,
  TimeCoordinate,
  TypeCoordinate,
  SpaceCoordinate,
  StateTransition,
  OntologyApiResponse,
} from "./types";

// ── 基础 API 调用 ───────────────────────────────────────────────────────────

const ONTOLOGY_BASE = "/api/ontology";

function ontologyUrl(table: string, id?: number, suffix?: string): string {
  if (id === undefined) return `${ONTOLOGY_BASE}/${table}`;
  if (suffix) return `${ONTOLOGY_BASE}/${table}/${id}/${suffix}`;
  return `${ONTOLOGY_BASE}/${table}/${id}`;
}

async function fetchOntology<T>(url: string): Promise<T> {
  const res = await apiClient.get<OntologyApiResponse<T>>(url);
  if (!res.success) throw new Error("Ontology API error");
  return res.data;
}

async function postOntology<T>(url: string, body: unknown): Promise<T> {
  const res = await apiClient.post<OntologyApiResponse<T>>(url, body);
  if (!res.success) throw new Error("Ontology API error");
  return res.data;
}

// ── useEntityView: 获取实体的完整本体业务表达 ───────────────────────────────

export function useEntityView(
  table: string,
  id?: number,
  options?: { enabled?: boolean }
) {
  return useQuery<OntologyExpression>({
    queryKey: ["ontology", "expression", table, id],
    queryFn: () => fetchOntology(ontologyUrl(table, id)),
    enabled: options?.enabled !== false && id !== undefined,
  });
}

// ── useEntityCoordinate: 获取实体的本体坐标 ─────────────────────────────────

export function useEntityCoordinate(
  table: string,
  id?: number,
  options?: { enabled?: boolean }
) {
  return useQuery<OntologyCoordinate>({
    queryKey: ["ontology", "coordinate", table, id],
    queryFn: () => fetchOntology(ontologyUrl(table, id, "coordinate")),
    enabled: options?.enabled !== false && id !== undefined,
  });
}

// ── useEntityTimeline: 获取实体的时间轴 ─────────────────────────────────────

export function useEntityTimeline(
  table: string,
  id?: number,
  options?: { enabled?: boolean }
) {
  return useQuery<TimeCoordinate>({
    queryKey: ["ontology", "timeline", table, id],
    queryFn: () => fetchOntology(ontologyUrl(table, id, "timeline")),
    enabled: options?.enabled !== false && id !== undefined,
  });
}

// ── useEntityInheritance: 获取实体的继承链 ──────────────────────────────────

export function useEntityInheritance(
  table: string,
  id?: number,
  options?: { enabled?: boolean }
) {
  return useQuery<TypeCoordinate>({
    queryKey: ["ontology", "inheritance", table, id],
    queryFn: () => fetchOntology(ontologyUrl(table, id, "inheritance")),
    enabled: options?.enabled !== false && id !== undefined,
  });
}

// ── useEntitySpace: 获取实体的空间坐标 ──────────────────────────────────────

export function useEntitySpace(
  table: string,
  id?: number,
  options?: { enabled?: boolean }
) {
  return useQuery<SpaceCoordinate>({
    queryKey: ["ontology", "space", table, id],
    queryFn: () => fetchOntology(ontologyUrl(table, id, "space")),
    enabled: options?.enabled !== false && id !== undefined,
  });
}

// ── useStateTransition: 状态转移 ────────────────────────────────────────────

export interface StateTransitionVariables {
  table: string;
  id: number;
  target_status_id: number;
}

export function useStateTransition() {
  const queryClient = useQueryClient();

  return useMutation<StateTransition, Error, StateTransitionVariables>({
    mutationFn: ({ table, id, target_status_id }) =>
      postOntology<StateTransition>(ontologyUrl(table, id, "state"), {
        target_status_id,
      }),
    onSuccess: (_, { table, id }) => {
      queryClient.invalidateQueries({
        queryKey: ["ontology", table, id],
      });
    },
  });
}

// ── usePhaseCheck: 生命周期阶段校验 ─────────────────────────────────────────

export interface PhaseCheckVariables {
  table: string;
  id: number;
  target_form: string;
  target_type: string;
}

export function usePhaseCheck() {
  return useMutation<{ allowed: boolean }, Error, PhaseCheckVariables>({
    mutationFn: ({ table, id, target_form, target_type }) =>
      postOntology<{ allowed: boolean }>(ontologyUrl(table, id, "check-phase"), {
        target_form,
        target_type,
      }),
  });
}

// ── useBatchEntityView: 批量投影 ────────────────────────────────────────────

export function useBatchEntityView(
  table: string,
  ids: number[],
  options?: { enabled?: boolean }
) {
  return useQuery<OntologyExpression[]>({
    queryKey: ["ontology", "batch", table, ids],
    queryFn: () =>
      postOntology<OntologyExpression[]>(ontologyUrl(table), { ids }),
    enabled: options?.enabled !== false && ids.length > 0,
  });
}
