/**
 * AliothEntityView — ontology-driven entity view.
 *
 * 摒弃从 DB → 后端 → 前端绑定映射 DB 字段的逻辑：
 *  - 后端 dispatcher 把 dk_scene/dk_factor/dk_function 三个绑定 + _refs
 *    关联解析一并返回
 *  - 前端只展示 (a) 通用字段 (notice/code/comments) (b) 关联引用 (c) 本体
 *    维度 (d) 其它业务字段按类型自适应（标量、文本、FK 引用）
 *  - 不存在「表 X 第 3 列第 4 行的 input」这种代码
 *
 * 用法：
 *   <AliothEntityView
 *     table="zc_id_agre-pricing"
 *     entityId={123}
 *     basePath="/gateway"
 *   />
 */

import * as React from "react";
import { useT } from "@alioth/i18n";
import {
 useOntologyEntity,
 type RefEntry,
 type AliothEntity,
} from "../hooks/useOntologyEntity";

export interface AliothEntityViewProps {
 table: string;
 entityId: number;
 basePath: string;
 /** 自定义字段渲染函数（按字段名） */
 fieldRenderer?: (key: string, value: unknown, refs: Record<string, RefEntry | RefEntry[] | null>) => React.ReactNode;
 /** 标题覆盖 */
 title?: (entity: AliothEntity) => string;
}

function isScalarValue(v: unknown): v is string | number | boolean {
 return typeof v === "string" || typeof v === "number" || typeof v === "boolean";
}

function isProtectedColumn(key: string): boolean {
 return [
  "id",
  "created_at",
  "updated_at",
  "created_by_id",
  "updated_by_id",
  "deleted_at",
  "deleted_by_id",
  "ak_dimensions",
  "ak_benefit_user",
  "ak_permit_user",
  "ak_access_user",
  "ak_source",
  "x_version",
  "tk_version",
  "tk_batch_no",
  "revision",
  "tpl_id",
  "majority",
  "sprint",
  "d_count",
  "number",
  "_refs",
 ].includes(key);
}

function RelationChip({ ref }: { ref: RefEntry | RefEntry[] | null }) {
 if (!ref) return <span className="text-muted-foreground">&mdash;</span>;
 if (Array.isArray(ref)) {
  return (
   <span className="inline-flex flex-wrap gap-1">
    {ref.map((r) => (
     <RelationChip key={r.id} ref={r} />
    ))}
   </span>
  );
 }
 const label = ref.notice || ref.code || `#${ref.id}`;
 const sub = ref.mark ?? ref.date;
 return (
  <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-muted text-muted-foreground">
   {label}
   {sub !== undefined && (
    <span className="text-[10px] opacity-70">({String(sub)})</span>
   )}
  </span>
 );
}

function DimensionBadge({ label, value }: { label: string; value?: number }) {
 if (!value) return null;
 return (
  <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-accent/20 text-accent-foreground">
   <span className="text-[10px] opacity-70">{label}</span>
   <span>{value}</span>
  </span>
 );
}

export function AliothEntityView({
 table,
 entityId,
 basePath,
 fieldRenderer,
 title,
}: AliothEntityViewProps) {
 const t = useT();
 const { data, refs, loading, error } = useOntologyEntity(table, entityId, { basePath });

 if (loading) {
  return (
   <div className="p-6 text-muted-foreground text-sm">
    {t("components.ontology.loading", undefined, { fallback: "Loading&hellip;" })}
   </div>
  );
 }
 if (error) {
  return (
   <div className="p-6 text-destructive text-sm">
    {t("components.ontology.error", undefined, { fallback: "Error" })}: {String(error.message ?? error)}
   </div>
  );
 }
 if (!data) {
  return (
   <div className="p-6 text-muted-foreground text-sm">
    {t("components.ontology.notFound", undefined, { fallback: "Entity not found" })}
   </div>
  );
 }

 const heading =
  title?.(data) ?? data.notice ?? data.code ?? `#${data.id}`;

 // 提取本体维度
 const dims = {
  scene: data.dk_scene,
  factor: data.dk_factor,
  function: data.dk_function,
 };

 // 提取业务字段
 const businessFields = Object.entries(data).filter(
  ([k, v]) =>
   !isProtectedColumn(k) &&
   v !== null &&
   v !== undefined &&
   v !== "" &&
   isScalarValue(v),
 );

 // 提取关联字段（fk_/qk_/sk_ 前缀）
 const relationFields = Object.entries(refs);

 return (
  <div className="max-w-5xl mx-auto p-6 space-y-4">
   {/* Header */}
   <header className="space-y-2">
    <h1 className="text-2xl font-semibold">{heading}</h1>
    <div className="flex flex-wrap gap-2">
     {data.t_color_ && (
      <span
       className="inline-block h-3 w-3 rounded-full"
       style={{ background: data.t_color_ }}
      />
     )}
     {data.code && (
      <span className="text-xs text-muted-foreground font-mono">
       {data.code}
      </span>
     )}
     {data.public !== undefined && (
      <span
       className={`inline-flex items-center px-2 py-0.5 rounded text-xs ${data.public
         ? "bg-success/20 text-success"
         : "bg-muted text-muted-foreground"
        }`}
      >
       {data.public ? "Public" : "Private"}
      </span>
     )}
    </div>
    {/* 本体维度 */}
    <div className="flex flex-wrap gap-1">
     <DimensionBadge label="Scene" value={dims.scene} />
     <DimensionBadge label="Factor" value={dims.factor} />
     <DimensionBadge label="Function" value={dims.function} />
    </div>
   </header>

   {/* 通用描述 */}
   {data.comments && (
    <section className="bg-card rounded-xl border p-4">
     <h2 className="text-sm font-medium text-muted-foreground mb-2">
      {t("components.ontology.comments", undefined, { fallback: "Comments" })}
     </h2>
     <p className="text-sm whitespace-pre-wrap">{data.comments}</p>
    </section>
   )}

   {/* 业务字段 — 不区分列名按类型自适应 */}
   {businessFields.length > 0 && (
    <section className="bg-card rounded-xl border p-4">
     <h2 className="text-sm font-medium text-muted-foreground mb-3">
      {t("components.ontology.attributes", undefined, { fallback: "Attributes" })}
     </h2>
     <dl className="grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-2">
      {businessFields.map(([k, v]) => (
       <div key={k} className="flex items-baseline gap-2 text-sm">
        <dt className="text-muted-foreground font-mono text-xs shrink-0">
         {k}
        </dt>
        <dd className="font-medium break-all">
         {fieldRenderer ? fieldRenderer(k, v, refs) : String(v)}
        </dd>
       </div>
      ))}
     </dl>
    </section>
   )}

   {/* 关联引用 — 不写字段名，按 _refs 整体渲染 */}
   {relationFields.length > 0 && (
    <section className="bg-card rounded-xl border p-4">
     <h2 className="text-sm font-medium text-muted-foreground mb-3">
      {t("components.ontology.relations", undefined, { fallback: "Relations" })}
     </h2>
     <div className="grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-2">
      {relationFields.map(([k, v]) => (
       <div key={k} className="flex items-baseline gap-2 text-sm">
        <span className="text-muted-foreground font-mono text-xs shrink-0">
         {k}
        </span>
        <RelationChip ref={v} />
       </div>
      ))}
     </div>
    </section>
   )}
  </div>
 );
}
