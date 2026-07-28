// @alioth/composables — 组合层业务组件
//
// 本包组合了 @alioth/components 的纯 UI 组件与
// @alioth/api / @alioth/hooks 的数据获取层，
// 提供"开箱即用"的业务组件。

// Block
export {
  registerBlock,
  registerBlockOrder,
  registerBlocks,
  getBlockComponent,
  getBlockOrder,
  useBlockRegistry,
  createBlockRoutes,
} from './block';
export type { BlockRegistration, BlockRouteMeta, BlockComponentMap, BlockNavKeyMap } from './block';

//
// 纯 UI 组件（无数据依赖）仍保留在 @alioth/components。

// Form
export { ReferenceSelect } from './form/ReferenceSelect';
export type { ReferenceSelectProps } from './form/ReferenceSelect';
export { AutoForm } from './form/AutoForm';
export type { AutoFormProps, FieldConfig } from './form/AutoForm';

// Schedule
export { useScheduleOverview } from './schedule/hooks';

// Workspace
export { UserProfilePanel } from './workspace/UserProfilePanel';
export { useWorkspaceOverview } from './workspace/useWorkspaceData';
export type { WorkspaceOverviewData } from './workspace/useWorkspaceData';
export { useWorkspaceSlots } from './workspace/useWorkspaceSlots';
export type { WorkspaceSlotConfig, WorkspaceSlotsResult } from './workspace/useWorkspaceSlots';

// Ontology — re-exported from @alioth/ontology (单一本体 seam)
export {
  useEntityView,
  useEntityCoordinate,
  useEntityTimeline,
  useEntityInheritance,
  useEntitySpace,
  useStateTransition,
  usePhaseCheck,
  useBatchEntityView,
  AliothEntityView,
} from '@alioth/ontology';
export type {
  StateTransitionVariables,
  PhaseCheckVariables,
  AliothEntityViewProps,
} from '@alioth/ontology';

// System Config
export { createSystemConfigPage } from './system-config/createSystemConfigPage';
export type { SystemConfigPageOptions } from './system-config/createSystemConfigPage';

// Layout
export { createModuleLayout } from './layout/createModuleLayout';
export type { ModuleLayoutOptions } from './layout/createModuleLayout';
export { ModuleLayout } from './layout/ModuleLayout';
export type {
  ModuleLayoutProps,
  ModuleNameConfig,
  ModuleWorkspaceConfig,
} from './layout/ModuleLayout';

// ── Re-exports from @alioth/components ──────────────────
// Module pages import these from composables for single-import convenience.
// Source: Framework/frontend/components/src/

// CRUD factories
export { createEntityListPage, createEntityTabbedListPage } from '@alioth/components';
export type {
  EntityListPageConfig,
  EntityListPageHooks,
  EntityListTabConfig,
  EntityTabbedListPageConfig,
  InlineEditingConfig,
  TabbedInlineEditingConfig,
  ConfigurableEntityListPage,
  ConfigurableEntityTabbedListPage,
} from '@alioth/components';

// UI Components — Card
export {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
  CardFooter,
  CardAction,
} from '@alioth/components';

// UI Components — Table
export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
} from '@alioth/components';

// UI Components — Stat/Status
export { StatGrid, StatCard } from '@alioth/components';
export type { StatCardProps, StatGridProps } from '@alioth/components';
export { StatusBadge } from '@alioth/components';
export type { StatusBadgeProps } from '@alioth/components';

// UI Components — Badges
export { OrderCateBadge } from '@alioth/components';

// Identity — shared typed hooks for personnel/role data
export {
  useEngineerList,
  useEngineerDetail,
  useSkillTagList,
  useApprovalRoleList,
  useCCBMemberList,
} from './identity';
export type { Engineer, SkillTag, ApprovalRole, CCBMember } from './identity';

export { EntityFormPage } from "./form/EntityFormPage";
export type { EntityFormPageProps } from "./form/EntityFormPage";
