/**
 * ApprovalPanel · 审批工作区面板
 *
 * 右侧滑出面板内的审批操作工作区。
 * 包含 Tab 切换、审批列表、搜索、空状态。
 */

import * as React from "react";
import { ClipboardCheck, Search } from "lucide-react";
import { cn } from "../../lib/utils";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { ScrollArea } from "../ui/scroll-area";
import { Input } from "../ui/input";
import { Skeleton } from "../ui/skeleton";
import { useT } from "@alioth/i18n"
import { EmptyState } from "../feedback/empty-state";
import { ApprovalCard } from "./ApprovalCard";
import type {
  ApprovalPanelProps,
  ApprovalTabId,
  ApprovalItem,
  ApprovalTab,
} from "./types";

function useApprovalTabs(): ApprovalTab[] {
  const t = useT();
  return [
    { id: "pending", label: t("components.approval.tab.pending") },
    { id: "approved", label: t("components.approval.tab.approved") },
    { id: "rejected", label: t("components.approval.tab.rejected") },
    { id: "my", label: t("components.approval.tab.my") },
  ];
}

/** 按 Tab 筛选审批项 */
function filterItems(
  items: ApprovalItem[],
  tab: ApprovalTabId,
  searchQuery: string,
): ApprovalItem[] {
  let filtered = items;

  // 按 Tab 筛选
  if (tab === "pending") {
    filtered = items.filter((i) => i.status === "pending");
  } else if (tab === "approved") {
    filtered = items.filter((i) => i.status === "approved");
  } else if (tab === "rejected") {
    filtered = items.filter((i) => i.status === "rejected");
  } else if (tab === "my") {
    // "我发起的"需要外部数据标记，这里简化为不过滤状态
    // 实际使用时建议传入时已过滤
    filtered = items;
  }

  // 按搜索词筛选
  if (searchQuery.trim()) {
    const q = searchQuery.toLowerCase();
    filtered = filtered.filter(
      (i) =>
        i.title.toLowerCase().includes(q) ||
        i.applicant.toLowerCase().includes(q) ||
        i.dept?.toLowerCase().includes(q) ||
        i.type?.toLowerCase().includes(q),
    );
  }

  return filtered;
}

/** 计算各 Tab 数量 */
function getTabCounts(items: ApprovalItem[]): Record<ApprovalTabId, number> {
  return {
    pending: items.filter((i) => i.status === "pending").length,
    approved: items.filter((i) => i.status === "approved").length,
    rejected: items.filter((i) => i.status === "rejected").length,
    my: items.length, // 简化处理，实际应由外部传入
  };
}

export const ApprovalPanel = React.forwardRef<HTMLDivElement, ApprovalPanelProps>(
  (
    {
      items,
      activeTab: controlledTab,
      onTabChange,
      onApprove,
      onReject,
      onItemClick,
      loading = false,
      className,
    },
    ref,
  ) => {
    const t = useT();
    const defaultTabs = useApprovalTabs();
    const [internalTab, setInternalTab] = React.useState<ApprovalTabId>("pending");
    const [searchQuery, setSearchQuery] = React.useState("");

    const activeTab = controlledTab ?? internalTab;
    const setActiveTab = (tab: ApprovalTabId) => {
      setInternalTab(tab);
      onTabChange?.(tab);
    };

    const counts = React.useMemo(() => getTabCounts(items), [items]);
    const filtered = React.useMemo(
      () => filterItems(items, activeTab, searchQuery),
      [items, activeTab, searchQuery],
    );

    const tabs = defaultTabs.map((t) => ({
      ...t,
      count: counts[t.id],
    }));

    return (
      <div ref={ref} className={cn("flex h-full flex-col", className)}>
        {/* 搜索栏 */}
        <div className="px-6 pt-4 pb-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              name="search"
              autoComplete="off"
              placeholder={t("components.approval.searchPlaceholder")}
              className="pl-9 text-sm"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* Tab 切换 */}
        <div className="px-6 pb-2">
          <Tabs
            value={activeTab}
            onValueChange={(v) => setActiveTab(v as ApprovalTabId)}
            className="w-full"
          >
            <TabsList className="w-full grid grid-cols-4 h-10 p-1 bg-muted">
              {tabs.map((tab) => (
                <TabsTrigger
                  key={tab.id}
                  value={tab.id}
                  className="text-xs font-medium relative"
                >
                  {tab.label}
                  {tab.count !== undefined && tab.count > 0 && (
                    <span
                      className={cn(
                        "ml-1 text-xs px-1 py-0 rounded-full",
                        activeTab === tab.id
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted-foreground/20 text-muted-foreground",
                      )}
                    >
                      {tab.count}
                    </span>
                  )}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </div>

        {/* 内容区 */}
        <div className="flex-1 overflow-hidden">
          <ScrollArea className="h-full px-6 py-2">
            {loading ? (
              <div className="space-y-3">
                {Array.from({ length: 4 }).map((_, i) => (
                  <Skeleton key={i} className="h-24 w-full rounded-xl" />
                ))}
              </div>
            ) : filtered.length > 0 ? (
              <div className="space-y-3 pb-4">
                {filtered.map((item) => (
                  <ApprovalCard
                    key={item.id}
                    item={item}
                    onApprove={onApprove}
                    onReject={onReject}
                    onClick={onItemClick}
                  />
                ))}
              </div>
            ) : (
              <EmptyState
                icon={<ClipboardCheck className="h-12 w-12" />}
                title={t("components.approval.empty.title")}
                description={
                  searchQuery
                    ? t("components.approval.empty.search")
                    : t("components.approval.empty.noData")
                }
              />
            )}
          </ScrollArea>
        </div>
      </div>
    );
  },
);

ApprovalPanel.displayName = "ApprovalPanel";
