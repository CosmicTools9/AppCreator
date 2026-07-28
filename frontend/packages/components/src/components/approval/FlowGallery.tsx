import { useMemo, useState, type ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Download, Layout, Plus, FileText } from "lucide-react";

export interface FlowItem {
  id: number;
  name: string;
  code?: string;
  status?: string;
  category?: string;
  "x-version"?: string;
  updated_at?: string;
  // 流程边拓扑（可选，后端未提供时回退序列渲染）
  edges?: Array<{ from: string; to: string }>;
}

export interface FlowTemplate<T = unknown> {
  id: string;
  name: string;
  desc: string;
  color: string;
  status: string;
  tplNodes: T[];
}

export interface FlowGalleryLabels {
  title: string;
  importMarkdown?: string;
  templateLibrary?: string;
  templateSubtitle?: string;
  createNew: string;
  empty: string;
  emptyHint: string;
  nodeMore?: string;
}
export interface FlowGalleryProps<T = unknown> {
  flows: FlowItem[];
  templates?: FlowTemplate<T>[];
  statuses: readonly string[];
  statusLabels: Record<string, string>;
  renderStatusBadge?: (status: string) => ReactNode;
  nodeTypeMeta: Array<{ type: string; label: string; color: string }>;
  flowIconResolver: (item: FlowItem) => { Icon: LucideIcon; cls: string };
  miniPreviewNodes: (flow: FlowItem) => Array<{
    id?: string;
    label: string;
    cls?: "start" | "end" | "condition" | "approval";
  }>;
  miniPreviewEdges?: (flow: FlowItem) => Array<{ from: string; to: string }>;
  templateNodeResolver: (node: T) => { id: string; type: string; label: string };
  labels: FlowGalleryLabels;
  onCreateNew: () => void;
  onSelectFlow: (flow: FlowItem) => void;
  onSelectTemplate?: (tpl: FlowTemplate<T>) => void;
  onImportMarkdown?: () => void;
  onImportTemplate?: () => void;
}

export interface GalleryMiniPreviewProps {
  flow: FlowItem;
  getNodes: (flow: FlowItem) => Array<{
    id?: string;
    label: string;
    cls?: "start" | "end" | "condition" | "approval";
  }>;
  getEdges?: (flow: FlowItem) => Array<{ from: string; to: string }>;
}

export function GalleryMiniPreview({ flow, getNodes, getEdges }: GalleryMiniPreviewProps) {
  const nodes = getNodes(flow);
  const edges = getEdges?.(flow);

  if (!edges || nodes.length === 0) {
    return (
      <div className="gallery-mini">
        {nodes.map((n, i) => (
          <span key={i} className="flex items-center gap-0.5">
            <span className={`gallery-mini-node ${n.cls ?? "approval"}`}>{n.label}</span>
            {i < nodes.length - 1 && <span className="gallery-mini-arrow">→</span>}
          </span>
        ))}
      </div>
    );
  }

  // 分支感知渲染：节点 id 缺省时以数组下标为隐式 id（向后兼容）
  const idOf = (i: number) => nodes[i].id ?? String(i);
  const idxById = new Map(nodes.map((_, i) => [idOf(i), i]));
  const outById: Record<string, string[]> = {};
  for (const e of edges) (outById[e.from] ??= []).push(e.to);

  const rendered = new Set<number>();
  const items: ReactNode[] = [];
  let cur = 0;
  let guard = 0;
  while (cur >= 0 && cur < nodes.length && guard++ < 40) {
    if (rendered.has(cur)) break;
    rendered.add(cur);
    const n = nodes[cur];
    const outs = (outById[idOf(cur)] ?? []).filter((id) => idxById.has(id));
    if (outs.length > 1) {
      const branches = outs.map((id) => idxById.get(id)!).slice(0, 4);
      branches.forEach((b) => rendered.add(b));
      items.push(
        <span key={cur} className="flex items-center gap-0.5">
          <span className={`gallery-mini-node ${n.cls ?? "approval"}`}>{n.label}</span>
          <span className="gallery-mini-arrow">→</span>
          <span className="gallery-mini-branch">
            {branches.map((b) => (
              <span key={b} className={`gallery-mini-node ${nodes[b].cls ?? "approval"}`}>
                {nodes[b].label}
              </span>
            ))}
          </span>
        </span>,
      );
      const nextIds = branches.map((b) => (outById[idOf(b)] ?? [])[0]).filter(Boolean);
      const converge = nextIds.length > 0 && nextIds.every((id) => id === nextIds[0]) ? nextIds[0] : undefined;
      const convergeIdx = converge !== undefined ? idxById.get(converge) : undefined;
      if (convergeIdx === undefined) break;
      items.push(<span key={`arr-${convergeIdx}`} className="gallery-mini-arrow">→</span>);
      cur = convergeIdx;
    } else {
      items.push(
        <span key={cur} className="flex items-center gap-0.5">
          <span className={`gallery-mini-node ${n.cls ?? "approval"}`}>{n.label}</span>
          {outs.length === 1 && <span className="gallery-mini-arrow">→</span>}
        </span>,
      );
      if (outs.length !== 1) break;
      cur = idxById.get(outs[0])!;
    }
    if (rendered.size >= 12) break;
  }

  const remaining = nodes.length - rendered.size;
  return (
    <div className="gallery-mini">
      {items}
      {remaining > 0 && <span className="gallery-mini-arrow">…</span>}
    </div>
  );
}

export function FlowGallery<T = unknown>({
  flows,
  templates,
  statuses,
  statusLabels,
  renderStatusBadge,
  nodeTypeMeta,
  flowIconResolver,
  miniPreviewNodes,
  miniPreviewEdges,
  templateNodeResolver,
  labels,
  onCreateNew,
  onSelectFlow,
  onSelectTemplate,
  onImportMarkdown,
  onImportTemplate,
}: FlowGalleryProps<T>) {
  const [sf, setSf] = useState(statuses[0] ?? "all");

  const filtered = useMemo(
    () => (sf === "all" || !sf ? flows : flows.filter((f) => f.status === sf)),
    [flows, sf]
  );

  const counts = useMemo(() => {
    const result: Record<string, number> = { all: flows.length };
    for (const s of statuses) {
      if (s !== "all") {
        result[s] = flows.filter((f) => f.status === s).length;
      }
    }
    return result;
  }, [flows, statuses]);


  return (
    <>
      {/* 标题 + 导入/模板按钮行 — 对齐 v33 prototype */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16 }}>
        <h1 className="text-lg font-semibold" style={{ flex: 1 }}>
          {labels.title}
        </h1>
        {onImportMarkdown && (
          <button type="button" onClick={onImportMarkdown} className="btn btn-ghost btn-sm">
            <Download className="w-3.5 h-3.5" /> {labels.importMarkdown}
          </button>
        )}
        {onImportTemplate && (
          <button type="button" onClick={onImportTemplate} className="btn btn-ghost btn-sm">
            <Layout className="w-3.5 h-3.5" /> {labels.templateLibrary}
          </button>
        )}
      </div>

      {flows.length === 0 && templates && templates.length > 0 ? (
        // 空数据时展示流程模板，对齐 v63 prototype
        <div className="flow-template-gallery">
          <div className="text-sm text-muted-foreground mb-4">
            {labels.templateSubtitle}
          </div>
          <div className="flex flex-wrap gap-4">
            {templates.map((tpl) => (
              <div
                key={tpl.id}
                onClick={() => onSelectTemplate?.(tpl)}
                className="bg-card border border-border rounded-lg p-4 cursor-pointer hover:border-primary/30 transition-colors"
                style={{ width: 300 }}
              >
                <div className="flex items-center gap-2 mb-2">
                  <span
                    style={{ width: 10, height: 10, borderRadius: "50%", background: tpl.color }}
                  />
                  <span className="font-semibold text-sm">{tpl.name}</span>
                  {renderStatusBadge ? renderStatusBadge(tpl.status) : tpl.status}
                </div>
                <p className="text-xs text-muted-foreground mb-2">{tpl.desc}</p>
                <div className="flex items-center gap-1 flex-wrap">
                  {tpl.tplNodes.slice(0, 5).map((n, i, arr) => {
                    const node = templateNodeResolver(n);
                    const meta = nodeTypeMeta.find((t) => t.type === node.type);
                    return (
                      <span key={node.id} className="flex items-center gap-1">
                        <span
                          className="px-1 py-1 rounded text-xs"
                          style={{
                            background: (meta?.color ?? "#999") + "18",
                            color: meta?.color ?? "#999",
                          }}
                        >
                          {node.label ?? meta?.label ?? node.type}
                        </span>
                        {i < arr.length - 1 && (
                          <span className="text-xs text-muted-foreground">→</span>
                        )}
                      </span>
                    );
                  })}
                  {tpl.tplNodes.length > 5 && (
                    <span className="text-xs text-muted-foreground">{labels.nodeMore ?? "..."}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <>
          {/* 过滤标签 + 新建流程 — 同一行（对齐 v33 prototype） */}
          <div className="filter-bar">
            {statuses.map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => setSf(s)}
                className={"filter-pill" + (sf === s ? " active" : "")}
              >
                {statusLabels[s] ?? s}
                <span className="count" style={{ fontSize: 10.5, opacity: 0.7 }}>
                  {counts[s] ?? 0}
                </span>
              </button>
            ))}
            <span style={{ flex: 1 }} />
            <button type="button" onClick={onCreateNew} className="btn btn-primary btn-sm">
              <Plus className="w-3.5 h-3.5" /> {labels.createNew}
            </button>
          </div>

          {filtered.length === 0 ? (
            <div className="empty-state">
              <div className="empty-state-icon">
                <FileText className="w-6 h-6 inline" />
              </div>
              <div className="text-15-600-mb6">{labels.empty}</div>
              <div
                style={{
                  fontSize: 13,
                  color: "hsl(var(--muted-foreground))",
                  maxWidth: 360,
                  margin: "0 auto 16px",
                }}
              >
                {labels.emptyHint}
              </div>
            </div>
          ) : (
            <div className="gallery-grid">
              {filtered.map((flow) => {
                const { Icon, cls } = flowIconResolver(flow);
                return (
                  <div
                    key={flow.id}
                    onClick={() => onSelectFlow(flow)}
                    className={"gallery-card" + (flow.status === "deprecated" ? " is-deprecated" : "")}
                  >
                    <div className="gallery-card-head">
                      <div className={"flow-icon " + cls}>
                        <Icon className="w-5 h-5" />
                      </div>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div className="gallery-card-name">{flow.name}</div>
                        <div className="gallery-card-code">{flow.code}</div>
                      </div>
                      {flow["x-version"] ? (
                        <div className="gallery-card-version">v{flow["x-version"]}</div>
                      ) : null}
                    </div>
                    <GalleryMiniPreview flow={flow} getNodes={miniPreviewNodes} getEdges={miniPreviewEdges} />
                    <div className="gallery-card-foot">
                      {renderStatusBadge ? renderStatusBadge(flow.status ?? "") : flow.status}
                      <span className="flow-date">
                        {flow.updated_at ? flow.updated_at.substring(0, 10) : ""}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </>
      )}
    </>
  );
}
