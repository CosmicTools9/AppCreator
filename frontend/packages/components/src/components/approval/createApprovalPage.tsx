//! 审批页面工厂
//!
//! 消除各模块 ApprovalPage 的复制粘贴。

import * as React from "react";
import { useNavigate } from "react-router";
import { ClipboardCheck, ArrowLeft } from "lucide-react";
import { ApprovalPanel } from "./ApprovalPanel";
import { useT } from "@alioth/i18n"
import type { ApprovalItem } from "./types";

export interface ApprovalPageOptions {
  /** 模块标识（如 "clients"、"vendors"），用于 i18n key */
  moduleName: string;
  /** 页面标题 i18n key，默认 `${moduleName}.approval.title` */
  titleKey?: string;
  /** 页面副标题 i18n key，默认 `${moduleName}.approval.subtitle` */
  subtitleKey?: string;
  /** 面板标题 i18n key，默认 `${moduleName}.approval.panelTitle` */
  panelTitleKey?: string;
  /** 标题 fallback */
  titleFallback?: string;
  /** 副标题 fallback */
  subtitleFallback?: string;
  /** 面板标题 fallback */
  panelTitleFallback?: string;
  /** 初始审批数据（可选，默认空数组） */
  initialItems?: ApprovalItem[];
}

/**
 * 创建模块审批页面
 *
 * @example
 * ```tsx
 * // pages/ApprovalPage.tsx
 * import { createApprovalPage } from "@aliothstudio/components/approval";
 *
 * export default createApprovalPage({
 *   moduleName: "clients",
 *   initialItems: [
 *     { id: 1, title: '...', applicant: '...', status: 'pending', time: '今天', type: '合同审批' },
 *   ],
 * });
 * ```
 */
export function createApprovalPage(options: ApprovalPageOptions) {
  const {
    moduleName,
    titleKey = `${moduleName}.approval.title`,
    subtitleKey = `${moduleName}.approval.subtitle`,
    panelTitleKey = `${moduleName}.approval.panelTitle`,
    titleFallback = "Approval Center",
    subtitleFallback = "Manage approval workflows",
    panelTitleFallback = "Approval List",
    initialItems = [],
  } = options;

  return function ApprovalPage(): React.ReactElement {
    const t = useT();
    const navigate = useNavigate();
    const [items, setItems] = React.useState<ApprovalItem[]>(initialItems);
    const [activeTab, setActiveTab] = React.useState<"pending" | "approved" | "rejected" | "my">("pending");

    const handleApprove = (id: string | number) => {
      setItems((prev) => prev.map((i) => (i.id === id ? { ...i, status: "approved" as const } : i)));
    };

    const handleReject = (id: string | number) => {
      setItems((prev) => prev.map((i) => (i.id === id ? { ...i, status: "rejected" as const } : i)));
    };

    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <button
              onClick={() => navigate("/")}
              className="w-9 h-9 rounded-lg flex items-center justify-center text-muted-foreground hover:bg-accent transition-colors cursor-pointer"
            >
              <ArrowLeft className="w-4 h-4" />
            </button>
            <div>
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                {t(titleKey, {}, { fallback: titleFallback })}
              </h1>
              <p className="text-sm text-muted-foreground mt-1">
                {t(subtitleKey, {}, { fallback: subtitleFallback })}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-card rounded-xl border">
          <div className="flex items-center gap-2 px-4 py-3 border-b">
            <ClipboardCheck className="w-4 h-4 text-primary" />
            <h3 className="text-sm font-semibold text-foreground">
              {t(panelTitleKey, {}, { fallback: panelTitleFallback })}
            </h3>
          </div>
          <div className="p-4">
            <ApprovalPanel
              items={items}
              activeTab={activeTab}
              onTabChange={setActiveTab}
              onApprove={handleApprove}
              onReject={handleReject}
              onItemClick={undefined}
            />
          </div>
        </div>
      </div>
    );
  };
}
