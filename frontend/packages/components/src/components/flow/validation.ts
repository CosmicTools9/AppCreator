import type { FlowNode, ValidationResult } from './types';
export function validateFlow(nodes: FlowNode[]): ValidationResult {
  const errors: Array<{ type: string; message: string; idx?: number }> = [];
  if (!nodes || nodes.length === 0) return { valid: false, errors: [{ type: 'empty', message: 'Flow cannot be empty' }] };
  const sIdxs: number[] = [], eIdxs: number[] = [];
  nodes.forEach((n, i) => { if (n.type === 'start') sIdxs.push(i); if (n.type === 'end') eIdxs.push(i); });
  if (sIdxs.length === 0) errors.push({ type: 'no_start', message: 'Missing start node' });
  if (sIdxs.length > 1) errors.push({ type: 'multi_start', message: 'Only one start node allowed', idx: sIdxs[0] });
  if (eIdxs.length === 0) errors.push({ type: 'no_end', message: 'Missing end node' });
  nodes.forEach((n, i) => {
    const nxt = n.next ?? [];
    if ((n.type === 'condition' || n.type === 'parallel') && nxt.length < 2) errors.push({ type: 'branch_missing', message: 'Branch needs >=2 outgoing edges', idx: i });
    nxt.forEach((e) => { if (e.to < 0 || e.to >= nodes.length) errors.push({ type: 'edge_oob', message: `Edge #${e.to} out of bounds`, idx: i }); });
  });
  const adj: number[][] = nodes.map((n) => (n.next ?? []).map((e) => e.to).filter((t) => t >= 0 && t < nodes.length));
  const vis: Record<number, boolean> = {}, rec: Record<number, boolean> = {};
  function dfs(u: number, p: number[]) { vis[u] = true; rec[u] = true; p.push(u); adj[u].forEach((v) => { if (!vis[v]) dfs(v, p); else if (rec[v]) { const c = p.slice(p.indexOf(v)).concat([v]); errors.push({ type: 'cycle', message: 'Cycle: #' + c.join(' → #') }); } }); p.pop(); rec[u] = false; }
  nodes.forEach((_, i) => { if (!vis[i]) dfs(i, []); });
  if (sIdxs.length > 0) {
    const reach: Record<number, boolean> = {}, q = [sIdxs[0]!]; reach[sIdxs[0]!] = true;
    while (q.length > 0) { const u = q.shift()!; adj[u].forEach((v) => { if (!reach[v]) { reach[v] = true; q.push(v); } }); }
    nodes.forEach((_, i) => { if (!reach[i]) errors.push({ type: 'unreachable', message: `Node #${i} unreachable from start`, idx: i }); });
  }
  return { valid: errors.length === 0, errors };
}
