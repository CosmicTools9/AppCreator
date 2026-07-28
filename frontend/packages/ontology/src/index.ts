//! Alioth Ontology Framework (Frontend)
//!
//! 提供 Alioth v10 本体坐标系统的前端类型和 Hooks。

export * from "./types";
export * from "./hooks";
export { EntityOntologyView } from "./components/EntityOntologyView";
export type { EntityOntologyViewProps } from "./components/EntityOntologyView";
export {
  useOntologyEntity,
  type RefEntry,
  type AliothEntity,
  type OntologyFetchResult,
  type UseOntologyEntityOptions,
} from "./hooks/useOntologyEntity";
export { AliothEntityView } from "./components/AliothEntityView";
export type { AliothEntityViewProps } from "./components/AliothEntityView";
