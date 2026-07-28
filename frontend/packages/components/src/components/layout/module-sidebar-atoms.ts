/**
 * Module Sidebar · 模块侧边栏内容跨 React Root 共享
 *
 * 当 ModuleLayout 在 embedded 模式下运行时，将导航项和品牌信息通过 `window`
 * 全局变量 + CustomEvent 传递给 Gateway 侧边栏。
 *
 * 之所以不使用 Jotai atom，是因为模块通过 MicroFrontendLoader 在独立的 React Root
 * 中渲染（createRoot），且每个模块是独立 Bundle（single-spa），Jotai atom 实例不共享。
 *
 * 用法：
 * - 模块端：setModuleSidebar(navItems, branding)
 * - Gateway 端：useModuleSidebar() → { navItems, branding }
 */

import * as React from "react";
import type { MainNavItem } from "./MainNav";

// ── Types ──────────────────────────────────────────

export interface ModuleSidebarBranding {
  title: string;
  subtitle: string;
  icon: string;
  /** 强调条颜色（CSS color 值），Gateway 用于同步模块主题 */
  accentBarColor?: string;
}

interface SidebarData {
  navItems: MainNavItem[];
  branding: ModuleSidebarBranding | null;
}


// ── Constants ──────────────────────────────────────

const WINDOW_KEY = "__ALIOTH_MODULE_SIDEBAR__";
const UPDATE_EVENT = "alioth-module-sidebar-update";

// ── Writers（模块端调用）───────────────────────────

/** 写入侧边栏数据（由模块 ModuleLayout 在 embedded 模式下调用） */
export function setModuleSidebar(
  navItems: MainNavItem[],
  branding: ModuleSidebarBranding,
): void {
  if (typeof window === "undefined") return;
  (window as unknown as Record<string, SidebarData>)[WINDOW_KEY] = {
    navItems,
    branding,
  };
  window.dispatchEvent(new CustomEvent(UPDATE_EVENT));
}

/** 清除侧边栏数据（模块卸载时调用） */
export function clearModuleSidebar(): void {
  if (typeof window === "undefined") return;
  delete (window as unknown as Record<string, SidebarData>)[WINDOW_KEY];
  window.dispatchEvent(new CustomEvent(UPDATE_EVENT));
}

// ── Reader ─────────────────────────────────────────

function getSidebarData(): SidebarData {
  if (typeof window === "undefined") {
    return { navItems: [], branding: null };
  }
  return (window as unknown as Record<string, SidebarData>)[WINDOW_KEY] ?? {
    navItems: [],
    branding: null,
  };
}

// ── Hook（Gateway 端调用）──────────────────────────

/**
 * 监听模块侧边栏数据变更。
 *
 * Gateway 的 Navigation 组件使用此 hook 读取模块推送的导航项和品牌信息。
 * 通过 CustomEvent 实现跨 React Root 响应式更新。
 */
export function useModuleSidebar(): SidebarData {
  const [data, setData] = React.useState<SidebarData>(getSidebarData);

  React.useEffect(() => {
    const handler = () => setData(getSidebarData());
    // 初始读取
    handler();
    window.addEventListener(UPDATE_EVENT, handler);
    return () => window.removeEventListener(UPDATE_EVENT, handler);
  }, []);

  return data;
}
