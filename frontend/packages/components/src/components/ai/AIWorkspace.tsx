/**
 * AIWorkspace · AI 助手工作区适配器
 *
 * 将 AIChatPanel 的面板内容嵌入 WorkspaceShell 统一外壳。
 * 不渲染浮动按钮，仅渲染对话面板。
 *
 * 增强（v2）：
 * - 通过 PageContextModule 获取渲染后的页面上下文
 * - 支持按 agentCode 渲染不同模板
 */

import * as React from "react";
import { useAtom } from "jotai";
import { AIChatPanel, type AIChatPanelProps, type PageContext } from "./AIChatPanel";
import { pageContextModule } from "./page-context";
import { aiContextAtom } from "./ai-context";
import { closeWorkspaceAtom } from "../workspace/workspace-atoms";

export interface AIWorkspaceProps
  extends Omit<
    AIChatPanelProps,
    "open" | "onOpenChange" | "docked" | "className" | "pageContext" | "contextMap"
  > {
  /** 当前绑定的 Agent 编码 */
  agentCode?: string;
  /** Agent 切换回调 */
  onAgentChange?: (agentCode: string) => void;
}

export const AIWorkspace = React.forwardRef<HTMLDivElement, AIWorkspaceProps>(
  ({ agentCode, onAgentChange, ...props }, ref) => {
    const [, close] = useAtom(closeWorkspaceAtom);
    // 订阅 aiContextAtom 以在上下文变化时触发重渲染
    const [aiState] = useAtom(aiContextAtom);
    const registeredAt = aiState.registeredAt;

    // 从 PageContextModule 读取并渲染 Agent 特定上下文
    const pageContext: PageContext | undefined = React.useMemo(() => {
      if (!registeredAt) return undefined;
      const envelope = pageContextModule.snapshot();
      const raw = envelope.raw;
      if (!raw) return undefined;

      const rendered = pageContextModule.renderForAgent(agentCode || "general");
      return {
        page: raw.page || raw.module || "Workbench",
        greeting: rendered.text,
        actions: raw.availableOperations || [],
        suggestedAgent: raw.suggestedAgent,
        intentHints: raw.intentHints,
      };
    }, [registeredAt, agentCode]);

    return (
      <div ref={ref} className="h-full">
        <AIChatPanel
          {...props}
          pageContext={pageContext}
          agentCode={agentCode}
          onAgentChange={onAgentChange}
          docked
          open={true}
          onOpenChange={(open) => !open && close()}
          panelClassName="border-0 bg-transparent"
        />
      </div>
    );
  },
);

AIWorkspace.displayName = "AIWorkspace";
