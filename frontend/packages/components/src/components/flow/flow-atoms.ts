import { atom } from 'jotai';
import type { FlowNode } from './types';
export type TabKey = 'pending' | 'history' | 'my-requests';
export const approvalTabAtom = atom<TabKey>('pending');
export const selectedApprovalIdAtom = atom<number | null>(null);
export type Screen =
  | { page: 'flow-gallery' }
  | { page: 'flow-designer-canvas'; id?: number; pendingTpl?: FlowNode[] }
  | { page: 'flow-form'; id?: number }
  | { page: 'flow-detail'; id: number }
  | { page: 'flow-publish'; id: number };
export const workflowDesignerScreenAtom = atom<Screen>({ page: 'flow-gallery' });
export const newRequestModalAtom = atom(false);
export const delegationModalAtom = atom(false);
export const pendingFilterAtom = atom<'all' | 'urgent' | 'high' | 'normal'>('all');
