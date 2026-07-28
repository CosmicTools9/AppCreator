//! Alioth Ontology 前端类型定义
//!
//! 对应后端 alioth_ontology crate 的坐标系统

// ── 空间坐标 ────────────────────────────────────────────────────────────────

export interface DimensionItem {
  id: number;
  notice: string;
  code?: string | null;
  color?: string | null;
}

export interface SpaceCoordinate {
  scene?: DimensionItem | null;
  factor?: DimensionItem | null;
  function?: DimensionItem | null;
}

// ── 时间坐标 ────────────────────────────────────────────────────────────────

export interface StateNode {
  id: number;
  notice: string;
  code?: string | null;
  color?: string | null;
  flag?: string | null; // "start" | "end" | etc.
}

export interface StateTransition {
  from?: StateNode | null;
  to: StateNode;
  triggered_at: string;
  triggered_by?: number | null;
}

export interface TimeCoordinate {
  current?: StateNode | null;
  all_states: StateNode[];
  available_transitions: StateNode[];
  history: StateTransition[];
}

// ── 类型坐标 ────────────────────────────────────────────────────────────────

export interface InheritanceNode {
  table_name: string;
  level: number;
  is_abstract: boolean;
}

export interface TypeCoordinate {
  path: InheritanceNode[];
  current_table: string;
  is_leaf: boolean;
  abstract_level: number;
}

// ── 完整本体坐标 ────────────────────────────────────────────────────────────

export interface OntologyCoordinate {
  space: SpaceCoordinate;
  time: TimeCoordinate;
  type_coord: TypeCoordinate;
}

// ── 本体业务表达 ────────────────────────────────────────────────────────────

export interface OntologyExpression {
  id: number;
  name: string;
  code?: string | null;
  phase: string;
  space_expression: string;
  status?: StateNode | null;
  categories: DimensionItem[];
  tags: DimensionItem[];
  inheritance: TypeCoordinate;
}

// ── API 响应包装 ────────────────────────────────────────────────────────────

export interface OntologyApiResponse<T> {
  success: boolean;
  data: T;
}

export interface OntologyApiError {
  success: false;
  error: string;
}
