//! ModuleLayout · 模块统一布局深组件
//!
//! 封装 Sidebar + MainNav + TopBar + ContentArea + WorkspaceDock 的完整布局逻辑。
//! 模块层只需提供 navItems、moduleName、workspaceConfig 和 children（Outlet）。

import * as React from "react";
import { useLocation, useNavigate, Link } from "react-router";
import { cn } from "@alioth/components";
import { DynamicIcon } from "@alioth/components";
import { MainNav, type MainNavItem } from "@alioth/components";
import { TopBar } from "@alioth/components";
import { ContentArea } from "@alioth/components";
import { useProvideAIContext, type AIPageContext } from "@alioth/components";
import {
  WorkspaceDock,
  WorkspaceTrigger,
  activeWorkspaceAtom,
} from "@alioth/components";
import { useWorkspaceSlots, nsId } from "../workspace/useWorkspaceSlots";
import type { InboxMessage, InboxSendParams } from "@alioth/components";
import type { ContactOption } from "@alioth/components";
import { useEmbedded } from "@alioth/components";
import { useAtom } from "jotai";
import { setModuleSidebar, clearModuleSidebar } from "@alioth/components";
import { useT } from "@alioth/i18n";
import {
  Search,
  PanelLeft,
  PanelRight,
  ClipboardCheck,
  Calendar,
  Mail,
  UserCircle,
  Bot,
  LayoutDashboard,
} from "lucide-react";

// ═══════════════════════════════════════════
// Types
// ═══════════════════════════════════════════

export interface ModuleWorkspaceConfig {
  /** 所属模块 ID — 用于命名空间化 workspace slot 和 trigger IDs，支持多模块各具独立能力 */
  moduleId?: string;
  /** 审批工作区 */
  approval?: {
    /** Block ID — 省略时使用默认审批 block-approval-execution */
    blockId?: string;
    onApprove?: (id: string | number) => void;
    onReject?: (id: string | number) => void;
  };
  /** 日程管理（仅启用/禁用） */
  schedule?: Record<string, never>;
  /** 站内信工作区 */
  inbox?: {
    onMessageClick?: (message: InboxMessage) => void;
    onDelete?: (id: string | number) => void;
    onMarkAllRead?: () => void;
    onReply?: (id: string | number, content: string) => void;
    onSend?: (params: InboxSendParams) => void;
    contacts?: ContactOption[];
  };
  ai?: {
    onSend: (message: string, pageContext?: unknown) => Promise<string>;
  };
}

export interface ModuleNameConfig {
  /** 英文标题（如 Process） */
  title: string;
  /** 中文描述（如 业务流程管理） */
  subtitle: string;
  /** 模块图标名称（Lucide 图标字符串，如 "Shield"） */
  icon: string;
  /** 顶部 Tab 配置（设置后取代面包屑作为上下文指示） */
  topTab?: { label: string; icon?: string };
  /** 强调条样式：solid = 主色实色，subtle = 15% 透明度（默认 subtle） */
  accentBarStyle?: "solid" | "subtle";
  /** 强调条自定义颜色（CSS color 值），优先级高于 accentBarStyle */
  accentBarColor?: string;
  /** 隐藏 TopBar 面包屑（设置 topTab 后自动隐藏） */
  hideBreadcrumb?: boolean;
  /** 隐藏模块品牌标识（Sidebar 品牌 block 已整体移除，此字段保留作为约定声明） */
  hideBrand?: boolean;
  /** 顶部搜索框 placeholder i18n key */
  topBarSearchPlaceholderKey?: string;
}


export interface ModuleLayoutProps {
  /** 导航项 */
  navItems: MainNavItem[];
  /** 模块名称配置 */
  moduleName: ModuleNameConfig;
  /** 右侧工作区配置 */
  workspaceConfig?: ModuleWorkspaceConfig;
  /** 是否显示顶部搜索框，默认 true */
  showSearch?: boolean;
  /** 集成模式：true 时不渲染 Sidebar/TopBar/WorkspaceDock，仅渲染内容区 */
  embedded?: boolean;
  /** 子内容（通常为 <Outlet />） */
  children?: React.ReactNode;
  /** 领域特定的 AI 上下文（快捷操作、业务上下文等） */
  aiContext?: Partial<AIPageContext>;
  /** 强制指定当前激活导航项 id（覆盖自动推断） */
  activeItemId?: string;
}

// ═══════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════

/** 从导航 href 提取基础路径，用于路由前缀匹配 */
function getNavBasePath(href: string): string {
  return href
    .replace(/\/(list|new|edit|detail|ontology)(\/.*)?$/, "")
    .replace(/\/$/, "");
}

/** 判断当前 pathname 是否匹配某导航项 */
function isNavActive(pathname: string, item: MainNavItem): boolean {
  if (pathname === item.href) return true;
  if (item.href === "/") return false;
  const base = getNavBasePath(item.href);
  if (!base) return false;
  return pathname === base || pathname.startsWith(base + "/");
}

/** 展平嵌套导航项，用于精确匹配和面包屑构建 */
function flattenNavItems(items: MainNavItem[]): MainNavItem[] {
  const result: MainNavItem[] = [];
  for (const item of items) {
    result.push(item);
    if (item.children) {
      result.push(...item.children);
    }
  }
  return result;
}

/** 查找当前 pathname 对应的最精确导航项（优先精确匹配 → 叶子节点 → 最长 href） */
function findActiveItem(pathname: string, items: MainNavItem[]): MainNavItem | undefined {
  const flat = flattenNavItems(items);
  const matches = flat.filter((item) => isNavActive(pathname, item));
  if (matches.length === 0) return undefined;
  return matches.sort((a, b) => {
    const aExact = a.href === pathname ? 1 : 0;
    const bExact = b.href === pathname ? 1 : 0;
    if (aExact !== bExact) return bExact - aExact;
    // 精确匹配相同时，优先叶子节点（无 children）
    const aLeaf = a.children ? 0 : 1;
    const bLeaf = b.children ? 0 : 1;
    if (aLeaf !== bLeaf) return bLeaf - aLeaf;
    return b.href.length - a.href.length;
  })[0];
}

/** 构建面包屑，支持嵌套导航 */
function buildBreadcrumbs(
  navItems: MainNavItem[],
  activeItem: MainNavItem | undefined,
  moduleName: ModuleNameConfig,
): Array<{ label: string; href?: string }> {
  const rootLabel = moduleName.topTab?.label ?? moduleName.title;
  const crumbs: Array<{ label: string; href?: string }> = [
    { label: rootLabel },
  ];
  if (!activeItem) return crumbs;

  // 检查 activeItem 是否是某个父项的子项
  for (const item of navItems) {
    if (item.children?.some((c) => c.id === activeItem.id)) {
      crumbs.push({ label: item.label });
      break;
    }
  }

  crumbs.push({ label: activeItem.label });
  return crumbs;
}

// ═══════════════════════════════════════════
// Sub-components
// ═══════════════════════════════════════════

function DefaultSearchSlot({ placeholder }: { placeholder?: string }) {
  const t = useT();
  const [search, setSearch] = React.useState("");
  return (
    <div className="relative w-72">
      <Search className="w-3.5 h-3.5 absolute left-[11px] top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
      <input
        type="text"
        placeholder={placeholder ?? t("moduleLayout.searchPlaceholder")}
        className="pl-9 pr-4 w-full h-[34px] text-sm border rounded-lg bg-muted focus:bg-background focus:outline-none focus:border-primary/30 focus:shadow-[0_0_0_3px_hsl(var(--primary)_/_0.08)] transition-colors"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />
    </div>
  );
}

// ═══════════════════════════════════════════
// Main Component
// ═══════════════════════════════════════════

export function ModuleLayout({
  navItems,
  moduleName,
  workspaceConfig,
  showSearch = true,
  embedded: embeddedProp,
  children,
  aiContext,
  activeItemId: activeItemIdProp,
}: ModuleLayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const t = useT();
  const [collapsed, setCollapsed] = React.useState(false);
  const [activeWorkspace] = useAtom(activeWorkspaceAtom);
  const isWorkspaceOpen = !!activeWorkspace;
  const workspaceRef = React.useRef<HTMLDivElement>(null);

  const accentBarStyle: React.CSSProperties = {
    height: 3,
    width: '100%',
    flexShrink: 0,
    backgroundColor: moduleName.accentBarColor
      ? moduleName.accentBarColor
      : moduleName.accentBarStyle === "solid"
        ? "hsl(var(--primary))"
        : "hsl(var(--primary) / 0.15)",
  };

  // 自动推断 Gateway basename（如 /members）
  const basePath = React.useMemo(() => {
    for (const item of navItems) {
      if (item.href !== "/" && item.href.startsWith("/") && location.pathname.includes(item.href)) {
        const idx = location.pathname.indexOf(item.href);
        if (idx >= 0) {
          return location.pathname.slice(0, idx);
        }
      }
    }
    return "";
  }, [location.pathname, navItems]);

  // 右侧面板打开时自动收起左侧边栏
  const prevCollapsedRef = React.useRef(false);
  const prevWorkspaceOpenRef = React.useRef(false);
  React.useEffect(() => {
    if (isWorkspaceOpen && !prevWorkspaceOpenRef.current) {
      prevCollapsedRef.current = collapsed;
      setCollapsed(true);
    } else if (!isWorkspaceOpen && prevWorkspaceOpenRef.current) {
      setCollapsed(prevCollapsedRef.current);
    }
    prevWorkspaceOpenRef.current = isWorkspaceOpen;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isWorkspaceOpen]);

  // 同步右侧面板宽度到 CSS 变量
  React.useEffect(() => {
    if (!isWorkspaceOpen || !workspaceRef.current) {
      document.documentElement.style.setProperty("--standard-drawer-right", "0px");
      return;
    }
    const el = workspaceRef.current;
    const update = () => {
      const width = el.getBoundingClientRect().width;
      document.documentElement.style.setProperty("--standard-drawer-right", `${width}px`);
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, [isWorkspaceOpen]);

  // 当前激活导航项
  const activeItem = React.useMemo(() => {
    if (activeItemIdProp) {
      const flat = flattenNavItems(navItems);
      const found = flat.find((item) => item.id === activeItemIdProp);
      if (found) return found;
    }
    return findActiveItem(location.pathname, navItems) ?? navItems[0];
  }, [activeItemIdProp, location.pathname, navItems]);

  const breadcrumbs = moduleName.hideBreadcrumb || moduleName.topTab
    ? []
    : buildBreadcrumbs(navItems, activeItem, moduleName);
  useProvideAIContext({
    module: moduleName.subtitle,
    page: activeItem?.label ?? "",
    availableOperations: navItems
      .filter((item) => item.id !== activeItem?.id && item.id !== "system-config")
      .map((item) => item.label),
    ...aiContext,
  });

  // 提取 moduleId 用于命名空间化
  const modId = workspaceConfig?.moduleId;

  // 统一构建 Workspace slots — 传入 moduleId 使 slot IDs 命名空间化
  const { slots, pendingCount, unreadCount } = useWorkspaceSlots({
    moduleId: modId,
    ai: workspaceConfig?.ai,
    approval: workspaceConfig?.approval,
    schedule: workspaceConfig?.schedule,
    inbox: workspaceConfig?.inbox,
    profile: { content: undefined },
  });

  // 检测是否在 Gateway 集成模式下运行
  const embeddedFromContext = useEmbedded();
  const embedded = embeddedProp ?? embeddedFromContext;
  // 集成模式：将导航项、品牌信息、主题色通过 window 跨 Root 推送给 Gateway
  React.useEffect(() => {
    if (embedded) {
      setModuleSidebar(navItems, {
        title: moduleName.title,
        subtitle: moduleName.subtitle,
        icon: moduleName.icon,
        accentBarColor: moduleName.accentBarColor,
      });
      return () => { clearModuleSidebar(); };
    }
  }, [embedded, navItems, moduleName.title, moduleName.subtitle, moduleName.icon, moduleName.accentBarColor]);

  // 集成模式：仅渲染内容区
  if (embedded) {
    return (
      <ContentArea
        padding="none"
        className="bg-muted/30 h-full"
        accentBar={
          <div
            className="accent-bar w-full shrink-0"
            style={accentBarStyle}
          />
        }
      >
        {children}
      </ContentArea>
    );
  }

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      {/* Sidebar */}
      <div
        className={cn(
          "flex flex-col border-r bg-secondary shrink-0 transition-[width] duration-300",
          collapsed ? "w-16" : "w-60"
        )}
      >
        <div className="flex-1 overflow-hidden">
          <MainNav
            items={navItems}
            activeItemId={activeItem?.id}
            onItemClick={(item) => {
              navigate(item.href);
            }}
            collapsed={collapsed}
            className="border-0"
          />
        </div>
        <div className="border-t p-3 flex items-center justify-center">
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="w-7 h-7 rounded-md flex items-center justify-center text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
            title={collapsed ? t("moduleLayout.expand") : t("moduleLayout.collapse")}
          >
            {collapsed ? <PanelRight className="w-3.5 h-3.5" /> : <PanelLeft className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Main content + Right Workspace wrapper */}
      <div className="flex-1 flex overflow-hidden">
        <div className={cn("flex flex-col overflow-hidden transition-[width] duration-300", "flex-1")}>
          <TopBar
            variant="module"
            breadcrumbs={breadcrumbs}
            tabs={moduleName.topTab ? [
              <button
                key="module-tab"
                className="flex items-center gap-1.5 px-3 h-8 text-sm font-semibold text-foreground bg-background border border-border rounded-md shadow-sm"
              >
                {moduleName.topTab.icon && (
                  <DynamicIcon name={moduleName.topTab.icon} className="w-4 h-4 text-primary" />
                )}
                {moduleName.topTab.label}
              </button>,
            ] : undefined}
            searchSlot={showSearch ? <DefaultSearchSlot placeholder={moduleName.topBarSearchPlaceholderKey ? t(moduleName.topBarSearchPlaceholderKey) : undefined} /> : undefined}
            actions={
              <div className="flex items-center gap-3">
                <Link
                  to="/"
                  className={cn(
                    "relative w-9 h-9 rounded-lg flex items-center justify-center transition-colors",
                    location.pathname === "/"
                      ? "bg-primary/10 text-primary"
                      : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                  )}
                  title={t("moduleLayout.workbench")}
                  aria-label={t("moduleLayout.workbench")}
                >
                  <LayoutDashboard className="w-4 h-4" />
                </Link>
                {/* 命名空间化 workspace triggers — 与 useWorkspaceSlots 生成的 slot ID 一致 */}
                {workspaceConfig?.ai && (
                  <WorkspaceTrigger
                    id={nsId(modId, "ai")}
                    icon={<Bot className="w-4 h-4" />}
                    title={t("moduleLayout.aiAssistant")}
                  />
                )}
                {workspaceConfig?.approval && (
                  <WorkspaceTrigger
                    id={nsId(modId, "approval")}
                    icon={<ClipboardCheck className="w-4 h-4" />}
                    title={t("moduleLayout.approvalFlow")}
                    pendingCount={pendingCount}
                  />
                )}
                {workspaceConfig?.schedule && (
                  <WorkspaceTrigger
                    id={nsId(modId, "schedule")}
                    icon={<Calendar className="w-4 h-4" />}
                    title={t("moduleLayout.scheduleManagement")}
                  />
                )}
                {workspaceConfig?.inbox && (
                  <WorkspaceTrigger
                    id={nsId(modId, "inbox")}
                    icon={<Mail className="w-4 h-4" />}
                    title={t("moduleLayout.inbox")}
                    unreadCount={unreadCount}
                  />
                )}
                <WorkspaceTrigger
                  id={nsId(modId, "profile")}
                  icon={<UserCircle className="w-4 h-4" />}
                  title={t("moduleLayout.userProfile")}
                />
              </div>
            }
          />
          <ContentArea
            padding="none"
            className="bg-muted/30"
            accentBar={
              <div
                className="accent-bar w-full shrink-0"
                style={accentBarStyle}
              />
            }
          >
            {children}
          </ContentArea>
        </div>
        {/* Right Docked Workspace */}
        {isWorkspaceOpen && slots.length > 0 && (
          <div ref={workspaceRef} className="w-96 h-full shrink-0">
            <WorkspaceDock slots={slots} />
          </div>
        )}
      </div>
    </div>
  );
}
