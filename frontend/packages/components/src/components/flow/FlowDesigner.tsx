/**
 * FlowDesigner — visual workflow designer component.
 * Used by both Alioth and AVIC-CAASEC system-approve module.
 */
import { useState, useCallback, useRef, useMemo, useEffect } from 'react';
import {
  Layout, X, Trash2, AlertTriangle, GitBranch,
  ArrowUp, ArrowDown, Undo2, Redo2, Save, RefreshCw,
} from 'lucide-react';
import type {
  FlowNode, FlowEdge, PortSide, NodeTypeConfig,
  FlowDesignerProps, ValidationResult,
} from './types';
import { simulateFlow } from './simulation';
import { validateFlow } from './validation';
import {
  effectiveNext, getNodeSize, ensurePositions, autoLayout,
  calcEndpoints, elbowPath, nodeColor,
  NODE_TYPES, NODE_W, NODE_H, PAD, COLS, X_GAP, Y_GAP,
} from './utils';
import { FlowNodePalette } from './FlowDesignerToolbar';

const DRAFT_KEY_PREFIX = 'flow.designer.';

// ── Node Card ─────────────────────────────────

function NodeCard({ node, idx, selected, multiSelected, highlighted, nodeTypes, onSelect, onPointerDown, onPortPointerDown, onContextMenu }: {
  node: FlowNode; idx: number; selected: boolean; multiSelected: boolean; highlighted: boolean;
  nodeTypes: NodeTypeConfig[];
  onSelect: (idx: number) => void;
  onPointerDown: (e: React.PointerEvent, idx: number) => void;
  onPortPointerDown: (e: React.PointerEvent, idx: number, side: PortSide) => void;
  onContextMenu: (e: React.MouseEvent, idx: number) => void;
}) {
  const nodeC = nodeColor(nodeTypes, node.type);
  const typeConf = nodeTypes.find((t) => t.type === node.type);
  const meta = node.type === 'approval' ? (node.mode === 'or_sign' ? '会签' : '或签')
    : node.type === 'condition' ? (node.expr || '未设置条件')
    : node.type === 'cc' ? (node.recipients || '')
    : node.type === 'parallel' ? `${node.branches ?? 2} 分支`
    : node.type === 'subflow' ? (node.target || '')
    : node.type === 'end' ? (node.outcome === 'reject' ? '驳回' : '完成')
    : '';
  return (
    <div
      className="vfd-node-wrapper"
      onPointerDown={(e) => {
        if ((e.target as HTMLElement).closest('[data-port]')) return;
        onSelect(idx);
        onPointerDown(e, idx);
      }}
      onContextMenu={(e) => onContextMenu(e, idx)}
      style={{
        position: 'absolute', left: node.x ?? 0, top: node.y ?? 0,
        width: getNodeSize(node.type).w, cursor: 'grab', zIndex: selected ? 20 : 10,
        borderRadius: '12px', border: selected ? '2px solid var(--primary)' : multiSelected ? '2px solid hsl(var(--primary)/0.5)' : '2px solid transparent',
        ...(highlighted ? { boxShadow: '0 0 0 3px hsl(var(--primary)/0.3)', transition: 'boxShadow 0.3s' } : {}),
      }}
    >
      <div
        className="vfd-node-head select-none"
        style={{
          display: 'flex', alignItems: 'center', gap: '8px',
          padding: '10px 12px', borderRadius: '10px 10px 0 0',
          background: `linear-gradient(135deg, ${nodeC}, ${nodeC}dd)`, color: '#fff',
        }}
      >
        <div style={{ width: 28, height: 28, borderRadius: '8px', background: 'rgba(255,255,255,0.2)', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
          <span className="text-xs font-bold">{idx + 1}</span>
        </div>
        <div className="flex-1 min-w-0">
          <div className="vfd-node-type" style={{ fontSize: 10, opacity: 0.8 }}>{typeConf?.label ?? node.type}</div>
          <div className="vfd-node-label" style={{ fontSize: 13, fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{node.label || typeConf?.label}</div>
        </div>
      </div>
      <div style={{ padding: '6px 12px', fontSize: 11, color: 'hsl(var(--muted-foreground))', background: 'hsl(var(--card))', borderRadius: '0 0 10px 10px', minHeight: 22 }}>
        {meta}
      </div>
      <div className="vfd-port vfd-port-top" data-port="top" style={{ position: 'absolute', top: -6, left: '50%', marginLeft: -6, width: 12, height: 12, borderRadius: '50%', background: 'hsl(var(--border))', border: '2px solid hsl(var(--card))', cursor: 'crosshair', zIndex: 5 }} onPointerDown={(e) => { e.stopPropagation(); onPortPointerDown(e, idx, 'top'); }} />
      <div className="vfd-port vfd-port-right" data-port="right" style={{ position: 'absolute', top: '50%', right: -6, marginTop: -6, width: 12, height: 12, borderRadius: '50%', background: 'hsl(var(--border))', border: '2px solid hsl(var(--card))', cursor: 'crosshair', zIndex: 5 }} onPointerDown={(e) => { e.stopPropagation(); onPortPointerDown(e, idx, 'right'); }} />
      <div className="vfd-port vfd-port-bottom" data-port="bottom" style={{ position: 'absolute', bottom: -6, left: '50%', marginLeft: -6, width: 12, height: 12, borderRadius: '50%', background: 'hsl(var(--border))', border: '2px solid hsl(var(--card))', cursor: 'crosshair', zIndex: 5 }} onPointerDown={(e) => { e.stopPropagation(); onPortPointerDown(e, idx, 'bottom'); }} />
      <div className="vfd-port vfd-port-left" data-port="left" style={{ position: 'absolute', top: '50%', left: -6, marginTop: -6, width: 12, height: 12, borderRadius: '50%', background: 'hsl(var(--border))', border: '2px solid hsl(var(--card))', cursor: 'crosshair', zIndex: 5 }} onPointerDown={(e) => { e.stopPropagation(); onPortPointerDown(e, idx, 'left'); }} />
    </div>
  );
}

// ── FlowDesigner ──────────────────────────────

export function FlowDesigner({
  initialNodes,
  flowName: initName,
  flowId,
  onSave,
  renderToolbar,
  renderInspector,
  onEnterSubflow,
  subflowStack,
  onExitSubflow,
  nodeTypeLabels,
  nodeTypes: customNodeTypes,
  draftRestoredLabel = 'A previously saved draft has been restored',
  discardDraftLabel = 'Discard draft',
}: FlowDesignerProps) {
  const resolvedNodeTypes = useMemo(() => {
    const base = customNodeTypes ?? NODE_TYPES;
    if (!nodeTypeLabels) return base;
    return base.map((nt) => {
      const o = nodeTypeLabels[nt.type];
      return o ? { ...nt, label: o.label, desc: o.desc } : nt;
    });
  }, [customNodeTypes, nodeTypeLabels]);

  const [name, setName] = useState(initName ?? 'Untitled Flow');
  useEffect(() => { setName(initName ?? 'Untitled Flow'); }, [initName]);

  const DRAFT_KEY = `${DRAFT_KEY_PREFIX}${flowId ?? 'new'}`;
  const restoredDraftRef = useRef(false);
  const [nodes, setNodes] = useState<FlowNode[]>(() => {
    const base = ensurePositions(initialNodes ?? [{ id: 'n-start', type: 'start', label: 'Start' }]);
    try {
      const raw = localStorage.getItem(DRAFT_KEY);
      if (raw) {
        const draft = JSON.parse(raw) as { nodes?: FlowNode[] };
        if (Array.isArray(draft.nodes) && draft.nodes.length > 0) {
          restoredDraftRef.current = true;
          return ensurePositions(draft.nodes);
        }
      }
    } catch { /* bad draft ignored */ }
    return base;
  });
  const [draftRestored, setDraftRestored] = useState(restoredDraftRef.current);

  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const [paletteOpen, setPaletteOpen] = useState(true);
  const [propOpen, setPropOpen] = useState(true);
  const [history, setHistory] = useState<FlowNode[][]>([]);
  const [future, setFuture] = useState<FlowNode[][]>([]);
  const [multiSel, setMultiSel] = useState<number[]>([]);
  const [zoom, setZoom] = useState(1);

  const dragRef = useRef<{ type: string } | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const freeDragRef = useRef<{ idx: number; startX: number; startY: number; origX: number; origY: number; wrapper: HTMLElement | null } | null>(null);

  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; idx: number | null;
    items: Array<{ label: string; icon: React.ReactNode; action: () => void; danger?: boolean; separator?: boolean }>;
  } | null>(null);
  const [edgeEdit, setEdgeEdit] = useState<{ from: number; edgeIdx: number } | null>(null);
  const [connecting, setConnecting] = useState<{ from: number; sx: number; sy: number; x: number; y: number } | null>(null);
  const [validation, setValidation] = useState<ValidationResult>({ valid: true, errors: [] });
  const [highlightedPath, setHighlightedPath] = useState<number[]>([]);
  const [draftDirty, setDraftDirty] = useState(false);

  // ── Auto-save draft ──
  useEffect(() => {
    if (!draftDirty) return;
    const id = setTimeout(() => {
      try { localStorage.setItem(DRAFT_KEY, JSON.stringify({ nodes, name, time: new Date().toISOString() })); } catch { /* ignore */ }
    }, 800);
    return () => clearTimeout(id);
  }, [nodes, name, draftDirty, DRAFT_KEY]);

  const pushHistory = useCallback((curr: FlowNode[]) => {
    setHistory((h) => [...h.slice(-20), JSON.parse(JSON.stringify(curr))]);
    setFuture([]);
    setDraftDirty(true);
  }, []);

  const undo = useCallback(() => {
    if (history.length === 0) return;
    setFuture((f) => [JSON.parse(JSON.stringify(nodes)), ...f]);
    const prev = history[history.length - 1]!;
    setHistory((h) => h.slice(0, -1));
    setNodes(prev);
  }, [history, nodes]);

  const redo = useCallback(() => {
    if (future.length === 0) return;
    setHistory((h) => [...h, JSON.parse(JSON.stringify(nodes))]);
    const next = future[0]!;
    setFuture((f) => f.slice(1));
    setNodes(next);
  }, [future, nodes]);

  const runValidation = useCallback(() => {
    const r = validateFlow(nodes);
    setValidation(r);
    return r.valid;
  }, [nodes]);

  const handleSave = useCallback(() => {
    const r = onSave(nodes, name);
    if (r && typeof (r as Promise<void>).then === 'function') {
      (r as Promise<void>).then(() => { setDraftDirty(false); try { localStorage.removeItem(DRAFT_KEY); } catch { /* ignore */ } }).catch(() => { /* keep dirty */ });
    } else {
      setDraftDirty(false);
      try { localStorage.removeItem(DRAFT_KEY); } catch { /* ignore */ }
    }
  }, [onSave, nodes, name, DRAFT_KEY]);

  const runSimulation = useCallback(() => {
    if (!runValidation()) return;
    const path = simulateFlow(nodes, {});
    setHighlightedPath(path);
    setTimeout(() => setHighlightedPath([]), 3000);
  }, [nodes, runValidation]);

  const relayout = useCallback(() => {
    pushHistory(nodes);
    setNodes((prev) => autoLayout(prev));
  }, [nodes, pushHistory]);

  // ── Keyboard shortcuts ──
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || (e.target as HTMLElement).isContentEditable) return;
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'z') { e.preventDefault(); undo(); }
      else if (((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z') || ((e.ctrlKey || e.metaKey) && e.key === 'y')) { e.preventDefault(); redo(); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [undo, redo]);

  // ── Node operations ──────────────────────────

  const updateNode = useCallback((idx: number, patch: Partial<FlowNode>) => {
    setNodes((prev) => { const arr = [...prev]; arr[idx] = { ...arr[idx]!, ...patch }; return arr; });
  }, []);

  const addNode = useCallback((type: string, baseX?: number, baseY?: number) => {
    const typeConf = resolvedNodeTypes.find((t) => t.type === type);
    pushHistory(nodes);
    const count = nodes.length;
    const newNode: FlowNode = {
      id: `n${Date.now()}`,
      type,
      label: typeConf?.label ?? type,
      x: baseX ?? PAD + (count % COLS) * X_GAP,
      y: baseY ?? PAD + Math.floor(count / COLS) * Y_GAP,
      ...(type === 'approval' ? { role: '', sla: 24, mode: 'or_sign' } : {}),
      ...(type === 'condition' ? { expr: '' } : {}),
      ...(type === 'cc' ? { recipients: '' } : {}),
      ...(type === 'end' ? { outcome: 'complete' } : {}),
      ...(type === 'parallel' ? { branches: 2 } : {}),
      ...(type === 'subflow' ? { target: '' } : {}),
    };
    setNodes((prev) => [...prev, newNode]);
    setSelectedIdx(count);
    setMultiSel([]);
  }, [nodes, pushHistory, resolvedNodeTypes]);

  const deleteNode = useCallback((idx: number) => {
    if (nodes.length <= 1) return;
    pushHistory(nodes);
    setNodes((prev) => prev.filter((_, i) => i !== idx));
    setSelectedIdx((s) => s === idx ? null : s != null && s > idx ? s - 1 : s);
    setMultiSel((prev) => prev.filter((i: number) => i !== idx).map((i: number) => i > idx ? i - 1 : i));
  }, [nodes, pushHistory]);

  const moveUp = useCallback((idx: number) => {
    if (idx <= 0) return;
    pushHistory(nodes);
    setNodes((prev) => { const arr = [...prev]; [arr[idx - 1], arr[idx]] = [arr[idx]!, arr[idx - 1]!]; return arr; });
    setSelectedIdx(idx - 1);
  }, [nodes, pushHistory]);

  const moveDown = useCallback((idx: number) => {
    if (idx >= nodes.length - 1) return;
    pushHistory(nodes);
    setNodes((prev) => { const arr = [...prev]; [arr[idx], arr[idx + 1]] = [arr[idx + 1]!, arr[idx]!]; return arr; });
    setSelectedIdx(idx + 1);
  }, [nodes, pushHistory]);

  const addBranch = useCallback((idx: number) => {
    if (!nodes[idx]) return;
    pushHistory(nodes);
    const cur = effectiveNext(nodes[idx]!, idx, nodes.length);
    const defaultTo = idx + 1 < nodes.length ? idx + 1 : idx;
    const nextArr: FlowEdge[] = [...cur, { to: defaultTo, label: cur.length === 0 ? '' : `branch ${cur.length + 1}` }];
    updateNode(idx, { next: nextArr });
  }, [nodes, pushHistory, updateNode]);

  // ── Drag from palette ───────────────────────

  const handlePaletteDragStart = useCallback((type: string) => {
    dragRef.current = { type };
  }, []);

  const handleCanvasDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    if (!dragRef.current || !canvasRef.current) return;
    const type = dragRef.current.type;
    dragRef.current = null;
    const rect = canvasRef.current.getBoundingClientRect();
    const x = (e.clientX - rect.left) / zoom;
    const y = (e.clientY - rect.top) / zoom;
    addNode(type, Math.max(0, x - NODE_W / 2), Math.max(0, y - NODE_H / 2));
  }, [addNode, zoom]);

  // ── Free drag on canvas ─────────────────────

  const handleNodePointerDown = useCallback((e: React.PointerEvent, idx: number) => {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('[data-port]') || target.closest('[data-check]')) return;
    e.preventDefault();
    e.stopPropagation();
    setSelectedIdx(idx);
    const wrapper = (e.currentTarget as HTMLElement).closest('.vfd-node-wrapper') as HTMLElement | null;
    if (wrapper) wrapper.style.cursor = 'grabbing';
    const n = nodes[idx];
    if (!n) return;
    freeDragRef.current = { idx, startX: e.clientX, startY: e.clientY, origX: n.x ?? 0, origY: n.y ?? 0, wrapper };

    const onMove = (me: PointerEvent) => {
      const fd = freeDragRef.current;
      if (!fd) return;
      const dx = (me.clientX - fd.startX) / zoom;
      const dy = (me.clientY - fd.startY) / zoom;
      setNodes((prev) => { const arr = [...prev]; arr[fd.idx] = { ...arr[fd.idx]!, x: Math.round(fd.origX + dx), y: Math.round(fd.origY + dy) }; return arr; });
    };
    const onUp = () => {
      const fd = freeDragRef.current;
      if (fd?.wrapper) fd.wrapper.style.cursor = 'grab';
      freeDragRef.current = null;
      document.removeEventListener('pointermove', onMove);
      document.removeEventListener('pointerup', onUp);
    };
    document.addEventListener('pointermove', onMove);
    document.addEventListener('pointerup', onUp);
  }, [nodes, zoom]);

  // ── Port connection ─────────────────────────

  const handlePortPointerDown = useCallback((e: React.PointerEvent, fromIdx: number, side: PortSide) => {
    e.preventDefault();
    e.stopPropagation();
    if (!canvasRef.current) return;
    const fromNode = nodes[fromIdx];
    if (!fromNode) return;
    const fromSize = getNodeSize(fromNode.type);
    const fx = fromNode.x ?? 0;
    const fy = fromNode.y ?? 0;
    const ports: Record<PortSide, { x: number; y: number }> = {
      top: { x: fx + fromSize.w / 2, y: fy },
      right: { x: fx + fromSize.w, y: fy + fromSize.h / 2 },
      bottom: { x: fx + fromSize.w / 2, y: fy + fromSize.h },
      left: { x: fx, y: fy + fromSize.h / 2 },
    };
    const sp = ports[side]!;
    const rect = canvasRef.current.getBoundingClientRect();
    setConnecting({ from: fromIdx, sx: sp.x, sy: sp.y, x: (e.clientX - rect.left) / zoom, y: (e.clientY - rect.top) / zoom });

    const onMove = (me: PointerEvent) => {
      if (!canvasRef.current) return;
      const r = canvasRef.current.getBoundingClientRect();
      setConnecting((c) => c ? { ...c, x: (me.clientX - r.left) / zoom, y: (me.clientY - r.top) / zoom } : null);
    };
    const onUp = (me: PointerEvent) => {
      document.removeEventListener('pointermove', onMove);
      document.removeEventListener('pointerup', onUp);
      setConnecting(null);
      if (!canvasRef.current) return;
      const r = canvasRef.current.getBoundingClientRect();
      const tx = (me.clientX - r.left) / zoom;
      const ty = (me.clientY - r.top) / zoom;
      let target = -1;
      nodes.forEach((n) => {
        if (n.id === fromNode.id) return;
        const size = getNodeSize(n.type);
        const nx = n.x ?? 0;
        const ny = n.y ?? 0;
        if (tx >= nx && tx <= nx + size.w && ty >= ny && ty <= ny + size.h) target = nodes.indexOf(n);
      });
      if (target >= 0) {
        pushHistory(nodes);
        setNodes((prev) => {
          const arr = [...prev];
          const src = arr[fromIdx]!;
          const cur = effectiveNext(src, fromIdx, arr.length);
          if (cur.some((edge) => edge.to === target)) return prev;
          arr[fromIdx] = { ...src, next: [...cur, { to: target, label: '' }] };
          return arr;
        });
      }
    };
    document.addEventListener('pointermove', onMove);
    document.addEventListener('pointerup', onUp);
  }, [nodes, pushHistory, zoom]);

  // ── Context menu ────────────────────────────

  const openNodeMenu = useCallback((e: React.MouseEvent, idx: number) => {
    e.preventDefault();
    e.stopPropagation();
    setSelectedIdx(idx);
    setContextMenu({
      x: e.clientX, y: e.clientY, idx,
      items: [
        { label: 'Move Up', icon: <ArrowUp className="w-3.5 h-3.5" />, action: () => moveUp(idx) },
        { label: 'Move Down', icon: <ArrowDown className="w-3.5 h-3.5" />, action: () => moveDown(idx) },
        { label: 'Add Branch', icon: <GitBranch className="w-3.5 h-3.5" />, action: () => addBranch(idx) },
        { separator: true, label: '', icon: null as any, action: () => {} },
        { label: 'Delete', icon: <Trash2 className="w-3.5 h-3.5" />, action: () => deleteNode(idx), danger: true },
      ],
    });
  }, [moveUp, moveDown, addBranch, deleteNode]);

  const openCanvasMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setContextMenu({
      x: e.clientX, y: e.clientY, idx: null,
      items: resolvedNodeTypes.map((nt) => ({
        label: `Add ${nt.label}`,
        icon: <span style={{ color: nt.color }}>{nt.label[0] ?? '?'}</span>,
        action: () => {
          if (!canvasRef.current) return;
          const rect = canvasRef.current.getBoundingClientRect();
          addNode(nt.type, Math.max(0, (e.clientX - rect.left) / zoom - NODE_W / 2), Math.max(0, (e.clientY - rect.top) / zoom - NODE_H / 2));
        },
      })),
    });
  }, [addNode, resolvedNodeTypes, zoom]);

  // ── Connectors ──────────────────────────────

  const layout = useMemo(() => {
    const positions: Record<number, { x: number; y: number }> = {};
    const sizes: Record<number, { w: number; h: number }> = {};
    let maxX = 0;
    let maxY = 0;
    nodes.forEach((n, i) => {
      const size = getNodeSize(n.type);
      const x = n.x ?? PAD + (i % COLS) * X_GAP;
      const y = n.y ?? PAD + Math.floor(i / COLS) * Y_GAP;
      positions[i] = { x: Math.round(x), y: Math.round(y) };
      sizes[i] = size;
      maxX = Math.max(maxX, x + size.w);
      maxY = Math.max(maxY, y + size.h);
    });
    return { positions, sizes, width: Math.max(maxX + PAD, 800), height: Math.max(maxY + PAD, 600) };
  }, [nodes]);

  const connections = useMemo(() => {
    const list: Array<{
      from: number; edgeIdx: number; sx: number; sy: number;
      ex: number; ey: number;
      fromSide: PortSide; toSide: PortSide;
      label?: string; cond?: string;
      isLoop: boolean; isBroken: boolean;
    }> = [];
    nodes.forEach((n, i) => {
      const fromSize = getNodeSize(n.type);
      const fromPos = layout.positions[i] ?? { x: n.x ?? 0, y: n.y ?? 0 };
      effectiveNext(n, i, nodes.length).forEach((edge, ei) => {
        const isLoop = edge.to === i;
        const isBroken = edge.to < 0 || edge.to >= nodes.length || !nodes[edge.to];
        const toPos = isBroken
          ? { x: fromPos.x + fromSize.w + 80, y: fromPos.y }
          : (layout.positions[edge.to] ?? { x: nodes[edge.to]?.x ?? 0, y: nodes[edge.to]?.y ?? 0 });
        const toSize = isBroken ? fromSize : getNodeSize(nodes[edge.to]!.type);
        const ends = calcEndpoints(fromPos, toPos, fromSize, toSize);
        list.push({
          from: i, edgeIdx: ei, sx: ends.sx, sy: ends.sy, ex: ends.ex, ey: ends.ey,
          fromSide: ends.fromSide, toSide: ends.toSide,
          label: edge.label, cond: edge.cond, isLoop, isBroken,
        });
      });
    });
    return list;
  }, [nodes, layout]);

  const edgeEditInfo = useMemo(() => {
    if (edgeEdit === null) return null;
    const node = nodes[edgeEdit.from];
    if (!node) return null;
    const allEdges = effectiveNext(node, edgeEdit.from, nodes.length);
    const edge = allEdges[edgeEdit.edgeIdx];
    if (!edge) return null;
    return { node, edge, from: edgeEdit.from, edgeIdx: edgeEdit.edgeIdx };
  }, [edgeEdit, nodes]);

  const handleDeleteEdge = useCallback(() => {
    if (!edgeEdit) return;
    pushHistory(nodes);
    setNodes((prev) => {
      const arr = [...prev];
      const src = arr[edgeEdit.from]!;
      const cur = effectiveNext(src, edgeEdit.from, arr.length);
      if (cur.length <= 1) {
        arr[edgeEdit.from] = { ...src, next: [] };
      } else {
        arr[edgeEdit.from] = { ...src, next: cur.filter((_, i) => i !== edgeEdit.edgeIdx) };
      }
      return arr;
    });
    setEdgeEdit(null);
  }, [edgeEdit, nodes, pushHistory]);

  // Zoom controls
  const zoomIn = useCallback(() => setZoom((z) => Math.min(2, z + 0.1)), []);
  const zoomOut = useCallback(() => setZoom((z) => Math.max(0.3, z - 0.1)), []);
  const fitToScreen = useCallback(() => { setZoom(1); }, []);

  // ── Render ──────────────────────────────────

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Toolbar */}
      <div className="shrink-0">
        {renderToolbar ? (
          renderToolbar({
            undo, redo, canUndo: history.length > 0, canRedo: future.length > 0,
            save: handleSave, relayout, togglePalette: () => setPaletteOpen((o) => !o), paletteOpen,
            toggleDrawer: () => setPropOpen((o) => !o), propOpen,
            validate: runValidation, simulate: runSimulation,
            validation, highlightedPath, draftDirty,
            zoom, zoomIn, zoomOut, fitToScreen,
            onOpenSubflow: onEnterSubflow, onExitSubflow, subflowStack,
          })
        ) : (
          <div className="vfd-topbar" style={{ padding: '8px 16px', borderBottom: '1px solid hsl(var(--border))', background: 'hsl(var(--card))' }}>
            <button type="button" onClick={() => setPaletteOpen((o) => !o)} className="btn btn-icon btn-sm" title="Node Palette">
              <Layout className="w-3.5 h-3.5" />
            </button>
            <div className="vfd-topbar-divider" />
            <button type="button" onClick={undo} disabled={history.length === 0} className="btn btn-ghost btn-sm" style={{ opacity: history.length > 0 ? 1 : 0.3 }} title="Undo">
              <Undo2 className="w-3.5 h-3.5" />
            </button>
            <button type="button" onClick={redo} disabled={future.length === 0} className="btn btn-ghost btn-sm" style={{ opacity: future.length > 0 ? 1 : 0.3 }} title="Redo">
              <Redo2 className="w-3.5 h-3.5" />
            </button>
            <div className="vfd-topbar-divider" />
            <button type="button" onClick={relayout} className="btn btn-ghost btn-sm" title="Auto Layout">
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
            <div className="vfd-topbar-divider" />
            <button type="button" onClick={handleSave} className="btn btn-primary btn-sm">
              <Save className="w-3 h-3" /> Save
            </button>
          </div>
        )}
      </div>

      {/* Draft restored banner */}
      {draftRestored ? (
        <div className="flex items-center gap-2 px-4 py-2 border-b shrink-0" style={{ background: 'hsl(var(--primary)/0.06)', borderColor: 'hsl(var(--primary)/0.2)' }}>
          <AlertTriangle className="w-4 h-4 shrink-0" style={{ color: 'hsl(var(--primary))' }} />
          <div className="text-xs flex-1">{draftRestoredLabel}</div>
          <button type="button" className="btn btn-ghost btn-sm" onClick={() => {
            try { localStorage.removeItem(DRAFT_KEY); } catch { /* ignore */ }
            setDraftRestored(false);
            setDraftDirty(false);
            setNodes(ensurePositions(initialNodes ?? [{ id: 'n-start', type: 'start', label: 'Start' }]));
          }}>{discardDraftLabel}</button>
        </div>
      ) : null}

      {/* Validation banner */}
      {!validation.valid ? (
        <div className="flex items-start gap-2 px-4 py-2 bg-destructive/5 border-b border-destructive/20 shrink-0">
          <AlertTriangle className="w-4 h-4 text-destructive shrink-0 mt-0.5" />
          <div className="text-xs text-destructive">
            <div className="font-semibold">Flow validation failed</div>
            {validation.errors.map((e, i) => (
              <div key={i} className="mt-0.5">{e.idx !== undefined ? `#${e.idx} ` : ''}{e.message}</div>
            ))}
          </div>
        </div>
      ) : null}

      {/* Content */}
      <div className="vfd-shell">
        {/* Palette */}
        <FlowNodePalette open={paletteOpen} nodeTypes={resolvedNodeTypes} onDragStart={handlePaletteDragStart} />

        {/* Canvas */}
        <div ref={canvasRef} className="vfd-canvas" onDragOver={(e) => e.preventDefault()} onDrop={handleCanvasDrop} onContextMenu={openCanvasMenu}
          style={{ overflow: 'auto', position: 'relative', flex: 1, background: 'hsl(var(--muted)/0.3)' }}>
          {nodes.length === 0 ? (
            <div className="vfd-canvas-empty" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'hsl(var(--muted-foreground))' }}>
              <Layout className="w-12 h-12" style={{ color: 'hsl(var(--muted-foreground)/0.4)' }} />
              <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 6 }}>Empty Canvas</div>
              <div style={{ fontSize: 13 }}>Drag nodes from the palette onto the canvas</div>
            </div>
          ) : (
            <div className="relative" style={{ width: layout.width, minHeight: layout.height, transform: `scale(${zoom})`, transformOrigin: '0 0' }}>
              {multiSel.length > 1 && (
                <div className="absolute top-2 left-2 z-30 flex items-center gap-1 px-2 py-1.5 rounded-lg bg-primary/5 border border-primary/20">
                  <span className="text-xs font-semibold text-primary mr-1">Selected {multiSel.length}</span>
                  <button type="button" onClick={() => { multiSel.slice().sort((a, b) => b - a).forEach((i: number) => deleteNode(i)); setMultiSel([]); }}
                    className="inline-flex items-center gap-1 px-2 py-1 rounded text-xs text-destructive hover:bg-destructive/10">
                    <Trash2 className="w-3 h-3" /> Delete
                  </button>
                </div>
              )}

              <svg className="vfd-connectors-svg" style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', pointerEvents: 'none', zIndex: 1 }}>
                <defs>
                  <marker id="arrowhead" viewBox="0 0 8 8" refX="8" refY="4" markerWidth="5" markerHeight="5" orient="auto">
                    <path d="M 0 0 L 8 4 L 0 8 z" fill="hsl(var(--primary))" />
                  </marker>
                </defs>
                {connections.map((c) => (
                  <g key={`conn-${c.from}-${c.edgeIdx}`}>
                    <path d={elbowPath(c.sx, c.sy, c.ex, c.ey, c.fromSide)}
                      className={`conn-line ${c.isLoop ? 'loop' : ''} ${c.isBroken ? 'broken' : ''}`}
                      style={{ fill: 'none', stroke: c.isLoop ? 'hsl(var(--warning))' : c.isBroken ? 'hsl(var(--destructive))' : 'hsl(var(--primary))', strokeWidth: 2, strokeDasharray: c.isBroken ? '2 6' : undefined }} />
                    <path d={elbowPath(c.sx, c.sy, c.ex, c.ey, c.fromSide)}
                      className="conn-hit"
                      style={{ fill: 'none', stroke: 'transparent', strokeWidth: 20, cursor: 'pointer', pointerEvents: 'stroke' }}
                      onClick={() => setEdgeEdit({ from: c.from, edgeIdx: c.edgeIdx })} />
                    {c.label && (
                      <text x={(c.sx + c.ex) / 2} y={Math.min(c.sy, c.ey) - 6} textAnchor="middle" fontSize={11} fill="hsl(var(--primary))" style={{ pointerEvents: 'none' }}>
                        {c.label}
                      </text>
                    )}
                  </g>
                ))}
                {connecting && (
                  <line x1={connecting.sx} y1={connecting.sy} x2={connecting.x} y2={connecting.y}
                    stroke="hsl(var(--primary)/0.5)" strokeWidth={2} strokeDasharray="4 4" />
                )}
              </svg>

              {nodes.map((n, i) => (
                <NodeCard
                  key={n.id} node={n} idx={i}
                  selected={selectedIdx === i}
                  multiSelected={multiSel.includes(i)}
                  highlighted={highlightedPath.includes(i)}
                  nodeTypes={resolvedNodeTypes}
                  onSelect={(idx) => { setSelectedIdx(idx); setPropOpen(true); }}
                  onPointerDown={handleNodePointerDown}
                  onPortPointerDown={handlePortPointerDown}
                  onContextMenu={openNodeMenu}
                />
              ))}
            </div>
          )}
        </div>

        {/* Properties drawer */}
        {selectedIdx != null && nodes[selectedIdx] && (
          <div className="vfd-drawer" style={{ width: propOpen ? 320 : 0, overflow: 'hidden', transition: 'width 0.2s', borderLeft: '1px solid hsl(var(--border))', background: 'hsl(var(--card))', flexShrink: 0 }}>
            <div style={{ minWidth: 320 }}>
              <div style={{ padding: '16px 20px', borderBottom: '1px solid hsl(var(--border))', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontWeight: 600, fontSize: 14 }}>Node Properties</span>
                <button onClick={() => setSelectedIdx(null)} className="btn btn-icon btn-sm"><X className="w-3.5 h-3.5" /></button>
              </div>
              <div style={{ padding: '16px 20px' }}>
                {renderInspector ? renderInspector({
                  node: nodes[selectedIdx]!,
                  onUpdate: (patch) => updateNode(selectedIdx, patch),
                }) : (
                  <div className="vfd-drawer-field" style={{ marginBottom: 12 }}>
                    <label style={{ display: 'block', fontSize: 12, fontWeight: 600, color: 'hsl(var(--muted-foreground))', marginBottom: 4 }}>Label</label>
                    <input type="text" value={nodes[selectedIdx]!.label} onChange={(e) => updateNode(selectedIdx, { label: e.target.value })}
                      className="input" placeholder="Node label" />
                  </div>
                )}
              </div>
              {edgeEditInfo && (
                <div style={{ borderTop: '1px solid hsl(var(--border))', padding: '16px 20px' }}>
                  <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 12 }}>Edge #{edgeEditInfo.edge.to}</div>
                  <div className="vfd-drawer-field" style={{ marginBottom: 12 }}>
                    <label style={{ display: 'block', fontSize: 12, fontWeight: 600, color: 'hsl(var(--muted-foreground))', marginBottom: 4 }}>Target</label>
                    <select value={edgeEditInfo.edge.to} onChange={(e) => {
                      const cur = effectiveNext(edgeEditInfo.node, edgeEditInfo.from, nodes.length);
                      cur[edgeEditInfo.edgeIdx] = { ...cur[edgeEditInfo.edgeIdx]!, to: Number(e.target.value) };
                      updateNode(edgeEditInfo.from, { next: cur });
                    }} className="input">
                      {nodes.map((n, i) => <option key={n.id} value={i}>#{i + 1} · {n.label}</option>)}
                    </select>
                  </div>
                  <div className="vfd-drawer-field" style={{ marginBottom: 12 }}>
                    <label style={{ display: 'block', fontSize: 12, fontWeight: 600, color: 'hsl(var(--muted-foreground))', marginBottom: 4 }}>Label</label>
                    <input type="text" value={edgeEditInfo.edge.label ?? ''} onChange={(e) => {
                      const cur = effectiveNext(edgeEditInfo.node, edgeEditInfo.from, nodes.length);
                      cur[edgeEditInfo.edgeIdx] = { ...cur[edgeEditInfo.edgeIdx]!, label: e.target.value };
                      updateNode(edgeEditInfo.from, { next: cur });
                    }} className="input" placeholder="e.g. Approved" />
                  </div>
                  <div className="vfd-drawer-field" style={{ marginBottom: 12 }}>
                    <label style={{ display: 'block', fontSize: 12, fontWeight: 600, color: 'hsl(var(--muted-foreground))', marginBottom: 4 }}>Condition</label>
                    <input type="text" value={edgeEditInfo.edge.cond ?? ''} onChange={(e) => {
                      const cur = effectiveNext(edgeEditInfo.node, edgeEditInfo.from, nodes.length);
                      cur[edgeEditInfo.edgeIdx] = { ...cur[edgeEditInfo.edgeIdx]!, cond: e.target.value };
                      updateNode(edgeEditInfo.from, { next: cur });
                    }} className="input" placeholder="e.g. amount > 5000" />
                  </div>
                  <div className="vfd-drawer-foot" style={{ display: 'flex', gap: 8 }}>
                    <button onClick={handleDeleteEdge} className="btn btn-sm" style={{ background: 'hsl(var(--destructive))', color: 'white', border: 'none' }}>Delete Edge</button>
                    <button onClick={() => setEdgeEdit(null)} className="btn btn-sm btn-outline">Close</button>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {contextMenu && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 90 }} onClick={() => setContextMenu(null)} />
          <div style={{ position: 'fixed', top: contextMenu.y, left: contextMenu.x, zIndex: 91, background: 'hsl(var(--card))', border: '1px solid hsl(var(--border))', borderRadius: 8, boxShadow: 'var(--shadow-lg)', padding: 4, minWidth: 160 }}>
            {contextMenu.items.map((item, i) =>
              item.separator ? <div key={i} style={{ height: 1, background: 'hsl(var(--border))', margin: '4px 0' }} />
              : (
                <button key={i} onClick={() => { item.action(); setContextMenu(null); }}
                  style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '6px 12px', fontSize: 13, width: '100%', border: 'none', background: 'transparent', cursor: 'pointer', borderRadius: 4, color: item.danger ? 'hsl(var(--destructive))' : 'hsl(var(--foreground))' }}
                  onMouseEnter={(e) => (e.currentTarget as HTMLElement).style.background = 'hsl(var(--accent))'}
                  onMouseLeave={(e) => (e.currentTarget as HTMLElement).style.background = 'transparent'}>
                  {item.icon} {item.label}
                </button>
              )
            )}
          </div>
        </>
      )}
    </div>
  );
}

export default FlowDesigner;
