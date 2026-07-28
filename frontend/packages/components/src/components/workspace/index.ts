// Workspace Components · 统一右侧工作区

export {
  activeWorkspaceAtom,
  openWorkspaceAtom,
  closeWorkspaceAtom,
  toggleWorkspaceAtom,
} from "./workspace-atoms";

export { WorkspaceShell } from "./WorkspaceShell";
export { WorkspaceHub } from "./WorkspaceHub";
export { WorkspaceDock } from "./WorkspaceDock";
export { WorkspaceTrigger } from "./WorkspaceTrigger";
// useWorkspaceSlots moved to @aliothstudio/composables

export type {
  WorkspaceId,
} from "./workspace-atoms";

export type {
  WorkspaceShellProps,
} from "./WorkspaceShell";

export type {
  WorkspaceSlot,
  WorkspaceHubProps,
} from "./WorkspaceHub";

export type {
  WorkspaceDockProps,
} from "./WorkspaceDock";

export type {
  WorkspaceTriggerProps,
} from "./WorkspaceTrigger";

// WorkspaceSlotConfig and WorkspaceSlotsResult moved to @aliothstudio/composables
