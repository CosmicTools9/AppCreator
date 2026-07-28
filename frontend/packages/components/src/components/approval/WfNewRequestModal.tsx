/**
 * WfNewRequestModal — 新建流程请求弹窗
 *
 * 通用框架级组件，不依赖模块级 API hooks 或 store atoms。
 * 所有数据依赖通过 props 注入，i18n labels 通过 props 接收。
 *
 * 简化版本：支持名称、流程选择、理由文本域。
 * FormSchemaBuilder 决策由 TG9 覆盖。
 */

import { useState } from 'react';
import { X } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';

export interface WfNewRequestLabels {
  title?: string;
  nameLabel?: string;
  namePlaceholder?: string;
  flowLabel?: string;
  flowPlaceholder?: string;
  reasonLabel?: string;
  reasonPlaceholder?: string;
  submitBtn?: string;
  cancelBtn?: string;
  noFlowsText?: string;
}

export interface WfNewRequestModalProps {
  /** 可选流程列表 */
  flows: Array<{
    id: number;
    name: string;
    status: string;
    formFields?: Array<{ key: string; label: string; type: string; required?: boolean }>;
  }>;
  /** 提交新建流程请求 */
  onSubmit: (data: { flowId: number; name: string; formData: Record<string, unknown> }) => void;
  /** 关闭弹窗 */
  onClose: () => void;
  /** 国际化标签（不传则用 English 默认值） */
  labels?: WfNewRequestLabels;
  /** 预填值（用于重新申请等场景） */
  initialName?: string;
  initialFlowId?: number;
  initialReason?: string;
  /** 可选：选中流程后渲染动态表单字段 */
  renderFormFields?: (() => React.ReactNode);
}

export function WfNewRequestModal({
  flows,
  onSubmit,
  onClose,
  labels,
  initialName,
  initialFlowId,
  initialReason,
  renderFormFields,
}: WfNewRequestModalProps) {
  const [name, setName] = useState(initialName ?? '');
  const [selectedFlowId, setSelectedFlowId] = useState<string>(
    initialFlowId ? String(initialFlowId) : '',
  );
  const [reason, setReason] = useState(initialReason ?? '');

  const t = {
    title: labels?.title ?? 'New Request',
    nameLabel: labels?.nameLabel ?? 'Request Name',
    namePlaceholder: labels?.namePlaceholder ?? 'Enter request name…',
    flowLabel: labels?.flowLabel ?? 'Flow',
    flowPlaceholder: labels?.flowPlaceholder ?? 'Select a flow…',
    reasonLabel: labels?.reasonLabel ?? 'Reason',
    reasonPlaceholder: labels?.reasonPlaceholder ?? 'Enter reason for this request…',
    submitBtn: labels?.submitBtn ?? 'Submit',
    cancelBtn: labels?.cancelBtn ?? 'Cancel',
    noFlowsText: labels?.noFlowsText ?? 'No flows available',
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedFlowId || !name.trim()) return;
    onSubmit({
      flowId: Number(selectedFlowId),
      name: name.trim(),
      formData: { reason: reason.trim() },
    });
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-card relative w-full max-w-lg rounded-xl border bg-background shadow-xl">
        {/* Modal Head */}
        <div className="modal-head flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold text-foreground">{t.title}</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Modal Body */}
        <form onSubmit={handleSubmit} className="modal-body space-y-5 px-6 py-5">
          {/* 请求名称 */}
          <div className="space-y-2">
            <Label htmlFor="request-name">{t.nameLabel}</Label>
            <Input
              id="request-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t.namePlaceholder}
              autoFocus
            />
          </div>

          {/* 流程选择 */}
          <div className="space-y-2">
            <Label htmlFor="flow-select">{t.flowLabel}</Label>
            <Select value={selectedFlowId} onValueChange={setSelectedFlowId}>
              <SelectTrigger id="flow-select" className="w-full">
                <SelectValue placeholder={t.flowPlaceholder} />
              </SelectTrigger>
              <SelectContent>
                {flows.length === 0 ? (
                  <SelectItem value="" disabled>
                    {t.noFlowsText}
                  </SelectItem>
                ) : (
                  flows.map((flow) => (
                    <SelectItem key={flow.id} value={String(flow.id)}>
                      {flow.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>

          {/* 理由 */}
          <div className="space-y-2">
            <Label htmlFor="request-reason">{t.reasonLabel}</Label>
            <Textarea
              id="request-reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder={t.reasonPlaceholder}
              rows={4}
              className="w-full resize-none"
            />
          </div>

          {/* 动态表单字段 */}
          {selectedFlowId && renderFormFields && (
            <div className="space-y-2">
              {renderFormFields()}
            </div>
          )}

          {/* Modal Foot */}
          <div className="flex items-center justify-end gap-2 pt-2">
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              {t.cancelBtn}
            </Button>
            <Button type="submit" size="sm" disabled={!name.trim() || !selectedFlowId}>
              {t.submitBtn}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
