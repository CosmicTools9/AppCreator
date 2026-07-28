/**
 * AI 上下文提供机制
 *
 * 设计目标：
 * - 当 AI 对话框在右侧边栏打开时，自动获取左侧主内容区的业务上下文
 * - 页面/模块通过 useProvideAIContext() 注册当前上下文
 * - AIWorkspace 通过 useAIContext() 读取上下文，注入到对话中
 *
 * 实现说明（2026-05-30 重构）：
 * - 内部委托给 PageContextModule 做验证、裁剪、渲染
 * - aiContextAtom 保留用于向后兼容与触发 React 重渲染
 * - 未来可逐步迁移到直接使用 pageContextModule
 */

import { atom, useAtom, useSetAtom } from "jotai";
import React from "react";
import { useLocation } from "react-router";
import {
  pageContextModule,
  type AIPageContext,
  type AIContextState,
} from "./page-context";

// 向后兼容：从 page-context 重新导出类型
export type { AIPageContext, AIContextState } from "./page-context";

// ═══════════════════════════════════════════
// Atom
// ═══════════════════════════════════════════

const EMPTY_CONTEXT: AIContextState = {
  pageContext: null,
  registeredAt: null,
};

/** @deprecated 未来可直接使用 pageContextModule；当前保留以兼容现有订阅 */
export const aiContextAtom = atom<AIContextState>(EMPTY_CONTEXT);
aiContextAtom.debugLabel = "aiContextAtom";

// ═══════════════════════════════════════════
// Hooks
// ═══════════════════════════════════════════

/**
 * 页面组件使用此 hook 向 AI 注册当前上下文。
 * 路由切换时自动重置，避免旧上下文残留。
 *
 * 使用模式：
 * - 在页面顶层组件中调用一次
 * - context 变化时（如筛选条件改变）自动更新
 */
export function useProvideAIContext(context: AIPageContext): void {
  const setContext = useSetAtom(aiContextAtom);
  const location = useLocation();

  React.useEffect(() => {
    pageContextModule.register(context);
    setContext({
      pageContext: context,
      registeredAt: Date.now(),
    });

    // 路由离开时自动清除上下文
    return () => {
      pageContextModule.clear();
      setContext(EMPTY_CONTEXT);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    context.module,
    context.page,
    context.currentData,
    context.recentActions,
    context.availableOperations,
    context.extraContext,
    context.suggestedAgent,
    context.intentHints,
    location.pathname,
    setContext,
  ]);
}

/**
 * AI 面板使用此 hook 读取当前上下文。
 * 返回结构化的上下文描述，可直接注入到 AI 对话中。
 *
 * 内部通过 PageContextModule.renderForAgent("general") 渲染 systemPrompt。
 */
export function useAIContext(): {
  hasContext: boolean;
  systemPrompt: string;
  raw: AIPageContext | null;
} {
  const [state] = useAtom(aiContextAtom);
  const ctx = state.pageContext;

  return React.useMemo(() => {
    if (!ctx) {
      return { hasContext: false, systemPrompt: "", raw: null };
    }

    const rendered = pageContextModule.renderForAgent("general");
    return {
      hasContext: true,
      systemPrompt: rendered.text,
      raw: ctx,
    };
  }, [ctx]);
}
