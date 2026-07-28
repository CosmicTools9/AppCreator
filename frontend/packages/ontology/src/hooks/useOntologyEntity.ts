/**
 * useOntologyEntity — low-level CRUD for the ontology dispatcher.
 *
 * The backend OntologyDispatcher exposes dynamic CRUD over any
 * zc_id_lifecycle leaf via:
 *   GET  /{module}/ontology/leaf/{table}
 *   GET  /{module}/ontology/leaf/{table}/{id}
 *
 * This hook abstracts page state for one entity.
 */

import { useEffect, useState, useCallback } from "react";
import { apiClient } from "@alioth/api";
import type { ApiResponse } from "@alioth/api";

export interface RefEntry {
 id: number;
 notice?: string;
 code?: string;
 /** 标量引用解析后的实际值 */
 mark?: number | string;
 date?: string;
 /** 原始 ZUID */
 zuid: number;
}

export interface AliothEntity {
 id: number;
 code?: string;
 notice?: string;
 comments?: string;
 o_number?: string;
 public?: boolean;
 domain_?: string;
  t_color_?: string;
 /** 本体维度绑定 */
 dk_scene?: number;
 dk_factor?: number;
 dk_function?: number;
 /** 任意业务字段（来自 leaf JSON 文档） */
 [key: string]: unknown;
 /** 关联引用解析结果 */
 _refs?: Record<string, RefEntry | RefEntry[] | null>;
}

export interface OntologyFetchResult {
 data: AliothEntity | null;
 refs: Record<string, RefEntry | RefEntry[] | null>;
 loading: boolean;
 error: Error | null;
 refetch: () => void;
}

export interface UseOntologyEntityOptions {
 /** 后端基础路径（如 "/gateway"） */
 basePath: string;
 /** 是否自动加载（默认 true） */
 enabled?: boolean;
}

/**
 * 加载单个 ontology 实体（含 _refs）。
 */
export function useOntologyEntity(
 table: string,
 id: number | null | undefined,
 options: UseOntologyEntityOptions,
): OntologyFetchResult {
 const { basePath, enabled = true } = options;
 const [data, setData] = useState<AliothEntity | null>(null);
 const [refs, setRefs] = useState<Record<string, RefEntry | RefEntry[] | null>>({});
 const [loading, setLoading] = useState(false);
 const [error, setError] = useState<Error | null>(null);
 const [tick, setTick] = useState(0);

 const refetch = useCallback(() => setTick((t) => t + 1), []);

 useEffect(() => {
  if (!enabled || id == null) return;
  let aborted = false;
  setLoading(true);
  setError(null);
  const url = `${basePath}/ontology/leaf/${encodeURIComponent(table)}/${id}`;
  apiClient
   .get<ApiResponse<{ data: AliothEntity; refs: Record<string, RefEntry | RefEntry[] | null> }>>(url)
   .then((resp) => {
    if (aborted) return;
    setData(resp.data?.data ?? null);
    setRefs(resp.data?.refs ?? {});
   })
   .catch((e) => {
    if (aborted) return;
    setError(e);
   })
   .finally(() => {
    if (!aborted) setLoading(false);
   });
  return () => { aborted = true; };
 }, [basePath, table, id, enabled, tick]);

 return { data, refs, loading, error, refetch };
}
