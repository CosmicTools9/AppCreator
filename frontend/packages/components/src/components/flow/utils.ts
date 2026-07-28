import type { FlowNode, NodeTypeConfig, PortSide } from './types';
export const NODE_TYPES: NodeTypeConfig[] = [
  { type: 'start', label: 'Start', desc: 'Flow trigger point', color: '#10b981' },
  { type: 'approval', label: 'Approval', desc: 'Single/multi approval', color: '#6366f1' },
  { type: 'condition', label: 'Condition', desc: 'Conditional routing', color: '#f59e0b' },
  { type: 'cc', label: 'CC', desc: 'Non-blocking notification', color: '#06b6d4' },
  { type: 'parallel', label: 'Parallel', desc: 'Multi-branch concurrency', color: '#8b5cf6' },
  { type: 'branch', label: 'Branch', desc: 'Merge point', color: '#ec4899' },
  { type: 'subflow', label: 'Subflow', desc: 'Reference another flow', color: '#14b8a6' },
  { type: 'end', label: 'End', desc: 'Flow terminator', color: '#94a3b8' },
];
export const NODE_W = 200, NODE_H = 72, PAD = 32, COLS = 4, X_GAP = 280, Y_GAP = 160;
export function effectiveNext(node: FlowNode, _idx: number, _total: number): NonNullable<FlowNode['next']> {
  const n = node.next; if (n && n.length > 0) return n; return [];
}
export function nodeIcon(type: string): string {
  const icons: Record<string, string> = { start: 'Play', end: 'Check', approval: 'Users', condition: 'Diamond', parallel: 'Layers', cc: 'Mail', branch: 'GitBranch', subflow: 'Layers' };
  return icons[type] ?? 'ChevronDown';
}

export function nodeColor(nt: NodeTypeConfig[], type: string): string { return nt.find((t) => t.type === type)?.color ?? '#6366f1'; }
export function getNodeSize(type: string): { w: number; h: number } {
  if (type === 'start') return { w: 56, h: 56 }; if (type === 'end') return { w: 60, h: 60 }; return { w: NODE_W, h: NODE_H };
}
export function ensurePositions(nodes: FlowNode[]): FlowNode[] {
  return nodes.length ? nodes.map((n, i) => ({ ...n, x: n.x ?? PAD + (i % COLS) * X_GAP, y: n.y ?? PAD + Math.floor(i / COLS) * Y_GAP })) : [];
}
export function autoLayout(nodes: FlowNode[]): FlowNode[] {
  if (nodes.length <= 1) return nodes.map((n) => ({ ...n }));
  const n = nodes.length, layer = new Array<number>(n).fill(0);
  for (let iter = 0; iter < n; iter++) for (let i = 0; i < n; i++) effectiveNext(nodes[i]!, i, n).forEach((e) => { if (e.to !== i && e.to >= 0 && e.to < n) layer[e.to] = Math.max(layer[e.to], layer[i]! + 1); });
  const maxL = Math.max(...layer, 0), byL: number[][] = Array.from({ length: maxL + 1 }, () => []);
  nodes.forEach((_, i) => byL[layer[i]!].push(i)); byL.forEach((c) => c.sort((a, b) => a - b));
  const colHs = byL.map((c) => Math.max(0, c.length - 1) * Y_GAP), maxH = Math.max(...colHs, 0);
  return nodes.map((nd, i) => { const l = layer[i]!, col = byL[l]!, j = col.indexOf(i); return { ...nd, x: PAD + l * X_GAP, y: PAD + (maxH - (Math.max(0, col.length - 1) * Y_GAP)) / 2 + j * Y_GAP }; });
}
export function calcEndpoints(from: { x: number; y: number }, to: { x: number; y: number }, fs: { w: number; h: number }, ts: { w: number; h: number }): { sx: number; sy: number; ex: number; ey: number; fromSide: PortSide; toSide: PortSide } {
  const dx = to.x - from.x, dy = to.y - from.y, adx = Math.abs(dx), ady = Math.abs(dy);
  let fromSide: PortSide, sx: number, sy: number;
  if (adx > ady) { fromSide = dx > 0 ? 'right' : 'left'; sx = dx > 0 ? from.x + fs.w : from.x; sy = from.y + fs.h / 2; }
  else { fromSide = dy > 0 ? 'bottom' : 'top'; sx = from.x + fs.w / 2; sy = dy > 0 ? from.y + fs.h : from.y; }
  let toSide: PortSide, ex: number, ey: number;
  const rdx = from.x - to.x, rdy = from.y - to.y, arx = Math.abs(rdx), ary = Math.abs(rdy);
  if (arx > ary) { toSide = rdx > 0 ? 'right' : 'left'; ex = rdx > 0 ? to.x + ts.w : to.x; ey = to.y + ts.h / 2; }
  else { toSide = rdy > 0 ? 'bottom' : 'top'; ex = to.x + ts.w / 2; ey = rdy > 0 ? to.y + ts.h : to.y; }
  return { sx, sy, ex, ey, fromSide, toSide };
}
export function elbowPath(sx: number, sy: number, ex: number, ey: number, fromSide?: string, _toSide?: string): string {
  if (fromSide === 'left' || fromSide === 'right') { const mx = (sx + ex) / 2; return `M ${sx} ${sy} L ${mx} ${sy} L ${mx} ${ey} L ${ex} ${ey}`; }
  const my = (sy + ey) / 2; return `M ${sx} ${sy} L ${sx} ${my} L ${ex} ${my} L ${ex} ${ey}`;
}
