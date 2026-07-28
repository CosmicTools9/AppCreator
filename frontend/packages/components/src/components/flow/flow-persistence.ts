import type { FlowNode } from './types';
export interface FlowMeta { category?: string; description?: string; }
export interface FlowGraphPayload { version: 1; nodes: FlowNode[]; meta?: FlowMeta; }
const NT: Record<string, true> = { start: true, approval: true, condition: true, cc: true, end: true, parallel: true, branch: true, subflow: true };
function isFN(v: unknown): v is FlowNode { if (typeof v !== 'object' || v === null) return false; const n = v as Record<string, unknown>; return typeof n.type === 'string' && NT[n.type] === true && typeof n.label === 'string'; }
function sanitize(v: unknown): FlowNode[] | null { return Array.isArray(v) ? (v.length === 0 ? [] : v.every(isFN) ? v : null) : null; }
function metaOf(v: unknown): FlowMeta | undefined {
  if (typeof v !== 'object' || v === null) return undefined;
  const m = v as Record<string, unknown>, meta: FlowMeta = {};
  if (typeof m.category === 'string') meta.category = m.category;
  if (typeof m.description === 'string') meta.description = m.description;
  return meta.category || meta.description ? meta : undefined;
}
export function serializeFlow(nodes: FlowNode[], meta?: FlowMeta): string {
  return JSON.stringify({ version: 1, nodes, ...(meta ? { meta } : {}) } as FlowGraphPayload);
}
export function deserializeFlow(raw: string | null | undefined): FlowGraphPayload | null {
  if (!raw) return null; let p: unknown;
  try { p = JSON.parse(raw); } catch { return null; }
  if (Array.isArray(p)) { const ns = sanitize(p); return ns ? { version: 1, nodes: ns } : null; }
  if (typeof p !== 'object' || p === null) return null;
  const o = p as Record<string, unknown>, ns = sanitize(o.nodes);
  if (!ns) return null;
  return { version: 1, nodes: ns, meta: metaOf(o.meta) };
}
