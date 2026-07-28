import type { FlowNode } from './types';
import { evaluateExpr } from './expression';
export function simulateFlow(nodes: FlowNode[], ctx: Record<string, unknown>): number[] {
  const path = [0]; let idx = 0;
  while (idx < nodes.length - 1) {
    const node = nodes[idx]; if (!node) break;
    const next = node.next ?? [];
    if (node.type === 'condition' && next.length > 1) {
      let chosen = false;
      for (const edge of next) {
        if (edge.cond) { const er = evaluateExpr(edge.cond, ctx); if (er.result) { idx = edge.to; chosen = true; break; } }
      }
      if (!chosen) idx = next[0]?.to;
    } else { idx = next[0]?.to; }
    if (idx == null || idx >= nodes.length || path.includes(idx)) break;
    path.push(idx);
  }
  return path;
}
