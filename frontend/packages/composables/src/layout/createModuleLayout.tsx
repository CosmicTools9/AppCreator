import * as React from "react";

//! 模块布局工厂
//!
//! 消除各模块 {Module}Layout 的复制粘贴。

import { Routes } from "react-router";
import { Outlet } from "react-router";
import { cn } from "@alioth/components";
import { useT } from "@alioth/i18n";
import type { TranslateFunction } from "@alioth/i18n";
import { ModuleLayout } from "./ModuleLayout";
import type { ModuleNameConfig } from "./ModuleLayout";
import type { MainNavItem } from "@alioth/components";
import type { AIPageContext } from "@alioth/components";
import { createAIChatService } from "@alioth/api";
import type { BlockComponentMap, BlockRouteMeta, BlockAssemblyConfig, BlockNavKeyMap } from "../block";
import { createBlockRoutes, deriveNavItems } from "../block";

/** Optional app-level props injected by Gateway at runtime. */
interface AliothAppProps {
  embedded?: boolean;
}

/** Narrow `window` access for the optional Gateway-injected `__ALIOTH_APP_PROPS__`. */
function readEmbeddedFlag(): boolean | undefined {
  if (typeof window === "undefined") return undefined;
  const props = (window as { __ALIOTH_APP_PROPS__?: AliothAppProps }).__ALIOTH_APP_PROPS__;
  return props?.embedded;
}

export interface ModuleLayoutOptions {
 /** 模块标识（如 "access"、"inventory"） */
 moduleName: string;
 /** 模块名称配置工厂（接收 t 函数，支持翻译） */
 getModuleConfig: (t: TranslateFunction) => ModuleNameConfig;
 /** 导航项工厂 hook（与 blockAssembly 二选一） */
 useNavItems?: () => MainNavItem[];
 /** AI 助手发送消息回调（可选） */
 onSend?: (message: string, pageContext?: unknown) => Promise<string>;
 /** 集成模式：true 时不渲染外壳，仅渲染内容区（由 Gateway 接管） */
 embedded?: boolean;
 /** 领域特定的 AI 上下文（快捷操作、业务上下文等） */
 aiContext?: Partial<AIPageContext>;
 /** AI 上下文 hook（在渲染时调用，可安全使用 hooks） */
 useAiContext?: () => Partial<AIPageContext>;
 useActiveItemId?: () => string | undefined;
 /** Block 组件映射 — 提供时根据 navItems 自动生成路由并填充全局 Block 注册表 */
 blockComponents?: BlockComponentMap;
 /** 选填的 Block 元数据（name, icon），传递给全局注册表 */
 blockMetas?: Record<string, BlockRouteMeta>;
 /** BlockAssembly 配置 — 提供时从 module.json#blockAssembly 自动派生 navItems */
 blockAssembly?: BlockAssemblyConfig;
 /** Block i18n key 映射 — blockAssembly 模式下必填（block ID / group ID → i18n key） */
 blockNavKeys?: BlockNavKeyMap;
}

/**
 * 创建模块布局组件
 *
 * 封装 ModuleLayout 的标准结构，消除各模块布局的复制粘贴。
 * AI 助手已整合到 TopBar（WorkspaceTrigger id="ai"），无需额外浮动按钮。
 *
 * @example
 * ```tsx
 * // components/AccessLayout.tsx
 * import { createModuleLayout } from "@alioth/components/layout";
 * import { useT } from "@alioth/i18n";
 *
 * function useAccessNavItems() {
 *   const t = useT();
 *   return [
 *     { id: 'dashboard', label: t('access.nav.dashboard'), href: '/', icon: "Shield" },
 *     // ...
 *   ];
 * }
 *
 * export const AccessLayout = createModuleLayout({
 *   moduleName: "access",
 *   moduleConfig: { title: "Access", subtitle: t => t('access.module.title'), icon: "Shield" },
 * });
 * ```
 */
export function createModuleLayout(options: ModuleLayoutOptions) {
 return function ModuleLayoutShell(): React.ReactNode {
  const t = useT();
  const resolvedAiContext = options.useAiContext?.() ?? options.aiContext;
  const activeItemId = options.useActiveItemId?.();

  const useNavItems = options.useNavItems;
  const hookNavItems = useNavItems?.() ?? [];
  const navItems = React.useMemo(() => {
   if (options.blockAssembly && options.blockNavKeys) {
    return deriveNavItems(options.blockAssembly, options.blockNavKeys, t);
   }
   return hookNavItems;
  }, [options.blockAssembly, options.blockNavKeys, hookNavItems, t]);

  const moduleNameConfig = options.getModuleConfig(t);

  const resolvedOnSend = React.useMemo(() => {
   if (options.onSend) return options.onSend;
   const aiChat = createAIChatService({});
   return async (message: string) => {
    const response = await aiChat.sendMessage(message);
    return response.content;
   };
  }, [options.onSend]);

  const embedded = options.embedded ?? readEmbeddedFlag();
  const sc = options.blockComponents;
  return (
   <div className={cn("flex flex-col h-full", `mod-${options.moduleName}`)}>
    <ModuleLayout
     navItems={navItems}
     moduleName={moduleNameConfig}
     embedded={embedded}
     aiContext={resolvedAiContext}
     activeItemId={activeItemId}
     workspaceConfig={{
      moduleId: options.moduleName,
      approval: {},
      schedule: {},
      inbox: {},
      ai: { onSend: resolvedOnSend },
     }}
    >
     {sc ? (
      <Routes>
       {createBlockRoutes(sc, navItems, options.blockMetas)}
      </Routes>
     ) : (
      <Outlet />
     )}
    </ModuleLayout>
   </div>
  );
 };
}
