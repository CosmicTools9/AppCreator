/**
 * InboxPanel · 站内信工作区面板
 *
 * 右侧滑出面板内的站内信操作工作区。
 * 包含搜索、Tab 切换、消息列表、消息详情视图、发送表单视图。
 * 列表、详情与发送在面板内切换，不跳出工作区。
 */

import * as React from "react";
import {
  Mail,
  Search,
  CheckCheck,
  PenLine,
} from "lucide-react";
import { cn } from "../../lib/utils";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { ScrollArea } from "../ui/scroll-area";
import { Input } from "../ui/input";
import { Button } from "../ui/button";
import { Skeleton } from "../ui/skeleton";
import { useT } from "@alioth/i18n";
import { EmptyState } from "../feedback/empty-state";
import { InboxMessageCard } from "./InboxMessageCard";
import { InboxMessageDetail } from "./InboxMessageDetail";
import { InboxSendForm } from "./InboxSendForm";
import type {
  InboxPanelProps,
  InboxTabId,
  InboxMessage,
  InboxTab,
} from "./types";

function useInboxTabs(): InboxTab[] {
  const t = useT();
  return [
    { id: "all", label: t("components.inbox.tab.all") },
    { id: "unread", label: t("components.inbox.tab.unread") },
    { id: "system", label: t("components.inbox.tab.system") },
  ];
}

/** 按 Tab 筛选消息 */
function filterMessages(
  messages: InboxMessage[],
  tab: InboxTabId,
  searchQuery: string,
): InboxMessage[] {
  let filtered = messages;

  // 按 Tab 筛选
  if (tab === "unread") {
    filtered = messages.filter((m) => m.unread);
  } else if (tab === "system") {
    filtered = messages.filter((m) => m.type === "system");
  }

  // 按搜索词筛选
  if (searchQuery.trim()) {
    const q = searchQuery.toLowerCase();
    filtered = filtered.filter(
      (m) =>
        m.title.toLowerCase().includes(q) ||
        m.content.toLowerCase().includes(q) ||
        m.from.toLowerCase().includes(q),
    );
  }

  return filtered;
}

/** 计算各 Tab 数量 */
function getTabCounts(messages: InboxMessage[]): Record<InboxTabId, number> {
  return {
    all: messages.length,
    unread: messages.filter((m) => m.unread).length,
    system: messages.filter((m) => m.type === "system").length,
  };
}

/** 视图模式 */
type InboxViewMode = "list" | "detail" | "compose";

export const InboxPanel = React.forwardRef<HTMLDivElement, InboxPanelProps>(
  (
    {
      messages,
      activeTab: controlledTab,
      onTabChange,
      onMessageClick,
      onDelete,
      onMarkAllRead,
      onReply,
      onSend,
      contacts = [],
      sending = false,
      loading = false,
      className,
    },
    ref,
  ) => {
    const t = useT();
    const defaultTabs = useInboxTabs();
    const [internalTab, setInternalTab] = React.useState<InboxTabId>("all");
    const [searchQuery, setSearchQuery] = React.useState("");
    const [selectedMessage, setSelectedMessage] =
      React.useState<InboxMessage | null>(null);
    const [viewMode, setViewMode] = React.useState<InboxViewMode>("list");

    const activeTab = controlledTab ?? internalTab;
    const setActiveTab = (tab: InboxTabId) => {
      setInternalTab(tab);
      onTabChange?.(tab);
    };

    // 外部 messages 变化时，若当前选中的消息已不存在，清空选中
    React.useEffect(() => {
      setSelectedMessage((prev) => {
        if (prev && !messages.find((m) => m.id === prev.id)) {
          return null;
        }
        return prev;
      });
    }, [messages]);

    const counts = React.useMemo(() => getTabCounts(messages), [messages]);
    const filtered = React.useMemo(
      () => filterMessages(messages, activeTab, searchQuery),
      [messages, activeTab, searchQuery],
    );

    const tabs = defaultTabs.map((t) => ({
      ...t,
      count: counts[t.id],
    }));

    const handleMessageClick = (message: InboxMessage) => {
      setSelectedMessage(message);
      setViewMode("detail");
      onMessageClick?.(message);
    };

    const handleBack = () => {
      setSelectedMessage(null);
      setViewMode("list");
    };

    const handleCompose = () => {
      setViewMode("compose");
    };

    const handleSend = (params: import("./types").InboxSendParams) => {
      onSend?.(params);
      // 发送成功后返回列表
      setViewMode("list");
    };

    const unreadCount = messages.filter((m) => m.unread).length;

    // ── 发送视图 ──
    if (viewMode === "compose") {
      return (
        <div ref={ref} className={cn("flex h-full flex-col", className)}>
          <InboxSendForm
            contacts={contacts}
            onBack={handleBack}
            onSend={handleSend}
            loading={sending}
          />
        </div>
      );
    }

    // ── 详情视图 ──
    if (viewMode === "detail" && selectedMessage) {
      return (
        <div ref={ref} className={cn("flex h-full flex-col", className)}>
          <InboxMessageDetail
            message={selectedMessage}
            onBack={handleBack}
            onDelete={(id) => {
              onDelete?.(id);
              setSelectedMessage(null);
              setViewMode("list");
            }}
            onReply={onReply}
          />
        </div>
      );
    }

    // ── 列表视图 ──
    return (
      <div ref={ref} className={cn("flex h-full flex-col", className)}>
        {/* 搜索栏 + 操作按钮 */}
        <div className="px-6 pt-4 pb-2 flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              name="search"
              autoComplete="off"
              placeholder={t("components.inbox.searchPlaceholder")}
              className="pl-9 text-sm"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          {unreadCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onMarkAllRead}
              className="gap-1 text-muted-foreground hover:text-foreground shrink-0"
              title={t("components.inbox.markAllRead")}
            >
              <CheckCheck className="w-4 h-4" />
              <span className="hidden sm:inline">{t("components.inbox.markAllRead")}</span>
            </Button>
          )}
          <Button
            size="sm"
            onClick={handleCompose}
            className="gap-1 shrink-0"
          >
            <PenLine className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">{t("components.inbox.compose", {}, { fallback: "写消息" })}</span>
          </Button>
        </div>

        {/* Tab 切换 */}
        <div className="px-6 pb-2">
          <Tabs
            value={activeTab}
            onValueChange={(v) => setActiveTab(v as InboxTabId)}
            className="w-full"
          >
            <TabsList className="w-full grid grid-cols-3 h-10 p-1 bg-muted">
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
                        "gl-1 text-xs px-1 py-0 rounded-full",
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
                {filtered.map((message) => (
                  <InboxMessageCard
                    key={message.id}
                    message={message}
                    onClick={handleMessageClick}
                    onDelete={onDelete}
                  />
                ))}
              </div>
            ) : (
              <EmptyState
                icon={<Mail className="h-12 w-12" />}
                title={t("components.inbox.empty.title")}
                description={
                  searchQuery
                    ? t("components.inbox.empty.search")
                    : t("components.inbox.empty.noData")
                }
              />
            )}
          </ScrollArea>
        </div>
      </div>
    );
  },
);

InboxPanel.displayName = "InboxPanel";
