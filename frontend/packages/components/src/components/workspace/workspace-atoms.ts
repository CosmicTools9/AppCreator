/**
 * Workspace Atoms · 右侧工作区全局状态
 *
 * 使用单个 atom 控制当前激活的右侧面板，天然互斥。
 * 所有 Workspace 组件共享此状态，确保一次只打开一个面板。
 */

import { atom } from "jotai";
import type { PrimitiveAtom } from "jotai";
import type { ReactNode } from "react";

/** 工作区标识 — 命名空间化以支持多模块各自的能力 */
export type WorkspaceId = string;

/** 工作区 Slot 定义（供 WorkspaceDock / useWorkspaceSlots 使用） */
export interface WorkspaceSlot {
  id: WorkspaceId;
  blockId: string;
  title: ReactNode;
  content: ReactNode;
}

/** 当前激活的工作区，null 表示全部关闭 */
export const activeWorkspaceAtom: PrimitiveAtom<WorkspaceId | null> = atom<WorkspaceId | null>(null);
activeWorkspaceAtom.debugLabel = "activeWorkspaceAtom";

/** 打开指定工作区（会关闭其他） */
export const openWorkspaceAtom = atom(
 null,
 (_get, set, id: WorkspaceId) => {
  set(activeWorkspaceAtom, id);
 },
);
openWorkspaceAtom.debugLabel = "openWorkspaceAction";

/** 关闭当前工作区 */
export const closeWorkspaceAtom = atom(null, (_get, set) => {
 set(activeWorkspaceAtom, null);
});
closeWorkspaceAtom.debugLabel = "closeWorkspaceAction";

/** 切换工作区（已打开则关闭，未打开则打开） */
export const toggleWorkspaceAtom = atom(
 null,
 (get, set, id: WorkspaceId) => {
  const current = get(activeWorkspaceAtom);
  set(activeWorkspaceAtom, current === id ? null : id);
 },
);
toggleWorkspaceAtom.debugLabel = "toggleWorkspaceAction";
