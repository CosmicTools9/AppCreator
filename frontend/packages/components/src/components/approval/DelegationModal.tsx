/**
 * DelegationModal — 审批委托弹窗
 *
 * 通用框架级组件，不依赖模块级 API hooks 或 store atoms。
 * 所有数据依赖通过 props 注入，i18n labels 通过 props 接收。
 */

import { useState } from "react";
import { X } from "lucide-react";
import { Button } from "../ui/button";
import { Textarea } from "../ui/textarea";
import { Label } from "../ui/label";
import { SearchSelect, type SearchSelectOption } from "../ui/search-select";

export interface DelegationModalLabels {
  title?: string;
  delegateToLabel?: string;
  delegateToPlaceholder?: string;
  scopeLabel?: string;
  saveBtn?: string;
  cancelBtn?: string;
}

export interface DelegationModalProps {
  /** 提交委托数据 */
  onSubmit: (data: { delegateTo: string; scope: string }) => void;
  /** 关闭弹窗 */
  onClose: () => void;
  /** 可委托人员选项（为空则使用文本输入） */
  delegateOptions?: SearchSelectOption[];
  /** 国际化标签（不传则用中文默认值） */
  labels?: DelegationModalLabels;
}

export function DelegationModal({
  onSubmit,
  onClose,
  labels,
  delegateOptions,
}: DelegationModalProps) {
  const [delegateTo, setDelegateTo] = useState("");
  const [scope, setScope] = useState("");

  const t = {
    title: labels?.title ?? "创建委托",
    delegateToLabel: labels?.delegateToLabel ?? "委托给",
    delegateToPlaceholder: labels?.delegateToPlaceholder ?? "请选择被委托人…",
    scopeLabel: labels?.scopeLabel ?? "委托范围",
    saveBtn: labels?.saveBtn ?? "保存",
    cancelBtn: labels?.cancelBtn ?? "取消",
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!delegateTo.trim()) return;
    onSubmit({ delegateTo: delegateTo.trim(), scope: scope.trim() });
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-card relative w-full max-w-md rounded-xl border bg-background shadow-xl">
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
          <div className="space-y-2">
            <Label htmlFor="delegate-to">{t.delegateToLabel}</Label>
            {delegateOptions !== undefined ? (
              <SearchSelect
                value={delegateTo}
                onValueChange={setDelegateTo}
                options={delegateOptions}
                placeholder={t.delegateToPlaceholder}
              />
            ) : (
              <input
                id="delegate-to"
                value={delegateTo}
                onChange={(e) => setDelegateTo(e.target.value)}
                placeholder={t.delegateToPlaceholder}
                autoFocus
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              />
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="delegation-scope">{t.scopeLabel}</Label>
            <Textarea
              id="delegation-scope"
              value={scope}
              onChange={(e) => setScope(e.target.value)}
              rows={4}
              className="w-full resize-none"
            />
          </div>

          {/* Modal Foot */}
          <div className="flex items-center justify-end gap-2 pt-2">
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              {t.cancelBtn}
            </Button>
            <Button type="submit" size="sm" disabled={!delegateTo.trim()}>
              {t.saveBtn}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
