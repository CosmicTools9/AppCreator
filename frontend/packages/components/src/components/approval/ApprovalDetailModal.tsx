/**
 * ApprovalDetailModal — 审批详情弹窗
 *
 * 通用框架级组件，不依赖模块级 API hooks 或 store atoms。
 * 所有数据依赖通过 props 注入，i18n labels 通过 props 接收。
 */

import { useState } from 'react';
import { Check, X, ArrowRight, Send, XCircle } from 'lucide-react';
import { Button } from '../ui/button';
import { Textarea } from '../ui/textarea';
import { ApproverPicker, type ApproverRef, type ApproverOption } from './ApproverPicker';
import { ApprovalNodeChain, type ChainNode } from './ApprovalNodeChain';
import { TimelineView, type TimelineNode } from './TimelineView';
import type { ApprovalItem } from './types';

/** 当前操作：审批/驳回/转办/抄送 */
type ModalAction = 'approve' | 'reject' | 'transfer' | 'cc' | null;

export interface ApprovalDetailModalProps {
  /** 审批单据数据 */
  item: ApprovalItem;
  /** 审批节点链 */
  chainNodes?: ChainNode[];
  /** 审批时间线事件 */
  timelineEvents?: TimelineNode[];
  /** 可选审批角色列表（用于转办/抄送） */
  approverRoles?: ApproverOption[];
  /** 可选审批人员列表（用于转办/抄送） */
  approverEngineers?: ApproverOption[];
  /** 审批通过 */
  onApprove: (opinion: string) => void;
  /** 驳回 */
  onReject: (opinion: string) => void;
  /** 转办 */
  onTransfer: (targetId: number, opinion?: string) => void;
  /** 抄送 */
  onCC: (targetId: number, opinion?: string) => void;
  /** 关闭弹窗 */
  onClose: () => void;
  /** 只读模式：隐藏操作面板，仅展示详情 */
  readOnly?: boolean;
  /** 国际化标签（不传则用 English 默认值） */
  labels?: {
    title?: string;
    approveBtn?: string;
    rejectBtn?: string;
    transferBtn?: string;
    ccBtn?: string;
    opinionPlaceholder?: string;
    transferTitle?: string;
    ccTitle?: string;
    confirmTransfer?: string;
    confirmCC?: string;
    cancelBtn?: string;
    /** 审批节点链 section 标题 */
    chainTitle?: string;
    /** 时间线 section 标题 */
    timelineTitle?: string;
    /** ApproverPicker 内嵌标签 */
    approverPicker?: {
      roleTab?: string;
      engineerTab?: string;
      selectPlaceholder?: string;
      searchPlaceholder?: string;
      emptyText?: string;
    };
  };
}

export function ApprovalDetailModal({
  item,
  chainNodes,
  timelineEvents,
  approverRoles = [],
  approverEngineers = [],
  onApprove,
  onReject,
  onTransfer,
  onCC,
  onClose,
  readOnly = false,
  labels,
}: ApprovalDetailModalProps) {
  const [action, setAction] = useState<ModalAction>(null);
  const [opinion, setOpinion] = useState('');
  const [target, setTarget] = useState<ApproverRef | null>(null);

  const t = {
    title: labels?.title ?? 'Approval Detail',
    approveBtn: labels?.approveBtn ?? 'Approve',
    rejectBtn: labels?.rejectBtn ?? 'Reject',
    transferBtn: labels?.transferBtn ?? 'Transfer',
    ccBtn: labels?.ccBtn ?? 'CC',
    opinionPlaceholder: labels?.opinionPlaceholder ?? 'Enter your opinion…',
    transferTitle: labels?.transferTitle ?? 'Transfer To',
    ccTitle: labels?.ccTitle ?? 'CC To',
    confirmTransfer: labels?.confirmTransfer ?? 'Confirm Transfer',
    confirmCC: labels?.confirmCC ?? 'Confirm CC',
    cancelBtn: labels?.cancelBtn ?? 'Cancel',
    chainTitle: labels?.chainTitle ?? 'Approval Chain',
    timelineTitle: labels?.timelineTitle ?? 'Timeline',
    approverPicker: {
      roleTab: labels?.approverPicker?.roleTab ?? 'Role',
      engineerTab: labels?.approverPicker?.engineerTab ?? 'Engineer',
      selectPlaceholder: labels?.approverPicker?.selectPlaceholder ?? 'Select target…',
      searchPlaceholder: labels?.approverPicker?.searchPlaceholder ?? 'Search…',
      emptyText: labels?.approverPicker?.emptyText ?? 'No results',
    },
  };

  const handlePrimaryAction = () => {
    if (action === 'approve') {
      onApprove(opinion);
    } else if (action === 'reject') {
      onReject(opinion);
    } else if (action === 'transfer' && target) {
      onTransfer(Number(target.id), opinion);
    } else if (action === 'cc' && target) {
      onCC(Number(target.id), opinion);
    }
    setAction(null);
    setOpinion('');
    setTarget(null);
  };

  const handleClose = () => {
    setAction(null);
    setOpinion('');
    setTarget(null);
    onClose();
  };

  const isTransferOrCC = action === 'transfer' || action === 'cc';
  const canConfirm =
    action === 'approve' || action === 'reject'
      ? true
      : isTransferOrCC
        ? target !== null && target.id !== ''
        : false;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => {
        if (e.target === e.currentTarget) handleClose();
      }}
    >
      <div className="modal-card relative max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded-xl border bg-background shadow-xl">
        {/* Modal Head */}
        <div className="modal-head flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-foreground">{t.title}</h2>
          <button
            type="button"
            onClick={handleClose}
            className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Modal Body */}
        <div className="modal-body space-y-6 px-6 py-5">
          {/* 单据概要 */}
          <div className="space-y-2">
            <h3 className="font-semibold text-foreground">{item.title}</h3>
            <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-muted-foreground">
              <span>
                {item.applicant}
                {item.dept ? ` · ${item.dept}` : ''}
              </span>
              <span>
                {item.code ? `${item.code} · ` : ''}
                {item.type ? `${item.type} · ` : ''}
                {item.time}
              </span>
            </div>
          </div>

          {/* 审批节点链 */}
          {chainNodes && chainNodes.length > 0 && (
            <div>
              <h4 className="mb-2 text-sm font-medium text-muted-foreground">{t.chainTitle}</h4>
              <ApprovalNodeChain nodes={chainNodes} />
            </div>
          )}

          {/* 时间线 */}
          {timelineEvents && timelineEvents.length > 0 && (
            <div>
              <h4 className="mb-2 text-sm font-medium text-muted-foreground">{t.timelineTitle}</h4>
              <TimelineView events={timelineEvents} />
            </div>
          )}

          {/* 操作面板 — readOnly 时隐藏 */}
          {!readOnly && (
            <div className="space-y-4 rounded-lg border bg-muted/30 p-4">
              {/* 审批意见 */}
              {action === 'approve' || action === 'reject' ? (
                <Textarea
                  placeholder={t.opinionPlaceholder}
                  value={opinion}
                  onChange={(e) => setOpinion(e.target.value)}
                  rows={3}
                  className="w-full resize-none"
                />
              ) : null}

              {/* 转办/抄送目标选择 */}
              {isTransferOrCC && (
                <div className="space-y-3">
                  <p className="text-sm font-medium text-foreground">
                    {action === 'transfer' ? t.transferTitle : t.ccTitle}
                  </p>
                  <ApproverPicker
                    value={target ?? undefined}
                    onChange={(ref) => setTarget(ref)}
                    roles={approverRoles}
                    engineers={approverEngineers}
                    labels={{
                      roleTab: t.approverPicker.roleTab,
                      engineerTab: t.approverPicker.engineerTab,
                      selectPlaceholder: t.approverPicker.selectPlaceholder,
                      searchPlaceholder: t.approverPicker.searchPlaceholder,
                      emptyText: t.approverPicker.emptyText,
                    }}
                  />
                  <Textarea
                    placeholder={t.opinionPlaceholder}
                    value={opinion}
                    onChange={(e) => setOpinion(e.target.value)}
                    rows={2}
                    className="w-full resize-none"
                  />
                </div>
              )}

              {/* 操作按钮组 — 未选择 action 时显示四按钮 */}
              {!action ? (
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    variant="default"
                    size="sm"
                    onClick={() => setAction('approve')}
                    className="bg-success text-success-foreground hover:bg-success/90"
                  >
                    <Check className="mr-1 h-4 w-4" />
                    {t.approveBtn}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setAction('reject')}
                    className="text-destructive hover:bg-destructive/10 hover:text-destructive"
                  >
                    <XCircle className="mr-1 h-4 w-4" />
                    {t.rejectBtn}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setAction('transfer')}
                    className="text-muted-foreground"
                  >
                    <ArrowRight className="mr-1 h-4 w-4" />
                    {t.transferBtn}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setAction('cc')}
                    className="text-muted-foreground"
                  >
                    <Send className="mr-1 h-4 w-4" />
                    {t.ccBtn}
                  </Button>
                </div>
              ) : (
                /* 确认/取消按钮组 */
                <div className="flex items-center gap-2">
                  <Button
                    variant="default"
                    size="sm"
                    disabled={!canConfirm}
                    onClick={handlePrimaryAction}
                  >
                    {action === 'transfer'
                      ? t.confirmTransfer
                      : action === 'cc'
                        ? t.confirmCC
                        : action === 'approve'
                          ? t.approveBtn
                          : t.rejectBtn}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setAction(null);
                      setOpinion('');
                      setTarget(null);
                    }}
                  >
                    {t.cancelBtn}
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Modal Foot */}
        <div className="modal-foot flex items-center justify-end gap-2 border-t px-6 py-3">
          <Button variant="outline" size="sm" onClick={handleClose}>
            {t.cancelBtn}
          </Button>
        </div>
      </div>
    </div>
  );
}
