import * as React from "react";
import { Bot, X, Send, Loader2 } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";

export interface PageContext {
  page: string;
  greeting: string;
  actions: string[];
  /** 建议的默认 Agent */
  suggestedAgent?: string;
  /** 意图提示词 */
  intentHints?: string[];
}

export interface AIMessage {
  role: "user" | "ai";
  text: string;
  /** 生成此消息的 Agent 编码 */
  agent_code?: string;
  /** 结构化数据（表单、分析结果等） */
  structured?: Record<string, unknown>;
  /** 是否需要用户确认 */
  requires_input?: boolean;
  /** 建议的操作按钮 */
  suggested_actions?: string[];
}

export interface AgentOption {
  code: string;
  name: string;
  /** Lucide 图标名 */
  icon?: string;
  /** UI 颜色 */
  color?: string;
  /** 分类 */
  category?: string;
}

export interface AIChatPanelProps {
  /**
   * 页面上下文配置，用于动态切换问候语和快捷操作。
   * 不传则根据 `window.location.pathname` 自动推断。
   */
  pageContext?: PageContext;

  /**
   * 自定义页面路径 → 上下文的映射表。
   * 默认内置工作台/日程/通讯录/消息/审批/档案 6 个页面上下文。
   */
  contextMap?: Record<string, PageContext>;

  /**
   * 浮动按钮的 bottom 位置，默认 `bottom-6`。
   */
  bottom?: string;

  /**
   * 浮动按钮的 left 位置，默认 `left-6`。
   * 与 `right` 互斥，同时传入时优先使用 `right`。
   */
  left?: string;

  /**
   * 浮动按钮的 right 位置。
   * 与 `left` 互斥，同时传入时优先使用本属性。
   */
  right?: string;

  /**
   * 发送消息回调。返回 Promise 时，组件会自动显示 loading 状态。
   * 返回字符串（纯文本）或 AIMessage（含结构化数据）。
   */
  onSend: (
    message: string,
    pageContext: PageContext,
  ) => Promise<string | AIMessage>;

  /**
   * AI 助手名称，默认 "AI 助手"。
   */
  assistantName?: string;

  /**
   * 输入框占位符模板。使用 `{page}` 替换页面名。
   * 默认：`在{page}场景下输入问题...`
   */
  placeholderTemplate?: string;

  /**
   * 自定义类名
   */
  className?: string;

  /**
   * 受控展开状态。传入后组件进入受控模式。
   */
  open?: boolean;

  /**
   * 展开状态变化回调。
   */
  onOpenChange?: (open: boolean) => void;

  /**
   * 嵌入布局模式。为 true 时不渲染浮动按钮，面板使用流式布局而非 fixed 定位。
   */
  docked?: boolean;

  /**
   * 面板容器自定义类名。用于在 WorkspaceShell 等统一外壳中嵌入时覆盖默认样式。
   */
  panelClassName?: string;

  /**
   * 当前 Agent 编码
   */
  agentCode?: string;

  /**
   * Agent 切换回调
   */
  onAgentChange?: (agentCode: string) => void;

  /**
   * 可用 Agent 列表（供选择器展示）
   */
  agents?: AgentOption[];
}

const DEFAULT_CONTEXT_MAP: Record<string, PageContext> = {
  dashboard: { page: "Workbench", greeting: "", actions: [] },
  calendar: { page: "Workbench", greeting: "", actions: [] },
  contacts: { page: "Workbench", greeting: "", actions: [] },
  messages: { page: "Workbench", greeting: "", actions: [] },
  approvals: { page: "Workbench", greeting: "", actions: [] },
  profile: { page: "Workbench", greeting: "", actions: [] },
};

function resolvePageContext(
  contextMap: Record<string, PageContext>,
  pageContext?: PageContext,
): PageContext {
  if (pageContext) return pageContext;

  const path = window.location.pathname.replace(/\/+$/, "");
  const segments = path.split("/");
  const lastSegment = segments[segments.length - 1] || "workbench";

  const segment = contextMap[lastSegment] || contextMap["workbench"] || {};
  return {
    page: segment.page || "Workbench",
    greeting: segment.greeting || "Hello! I'm on the Workbench.",
    actions: segment.actions || ["View todos", "Query inventory", "Arrange meetings"],
  };
}

interface DragPosition {
  left?: number;
  right?: number;
  bottom: number;
}

export const AIChatPanel = React.forwardRef<
  HTMLButtonElement,
  AIChatPanelProps
>(
  (
    {
      pageContext,
      contextMap = DEFAULT_CONTEXT_MAP,
      bottom = "bottom-6",
      left = "left-6",
      right,
      onSend,
      placeholderTemplate,
      className,
      open: openProp,
      onOpenChange,
      docked,
      panelClassName,
      agentCode,
      onAgentChange,
      agents = [],
    },
    ref,
  ) => {
    const [internalOpen, setInternalOpen] = React.useState(false);
    const isOpen = openProp !== undefined ? openProp : internalOpen;
    const setIsOpen = (value: boolean) => {
      if (openProp === undefined) setInternalOpen(value);
      onOpenChange?.(value);
    };

    const t = useT();
    const resolvedPlaceholderTemplate = placeholderTemplate ?? t("ai.placeholderTemplate");
    const [input, setInput] = React.useState("");
    const [messages, setMessages] = React.useState<AIMessage[]>([]);
    const [loading, setLoading] = React.useState(false);
    const [showAgentPicker, setShowAgentPicker] = React.useState(false);
    const scrollRef = React.useRef<HTMLDivElement>(null);
    const inputRef = React.useRef<HTMLInputElement>(null);
    const initializedRef = React.useRef(false);
    const innerRef = React.useRef<HTMLButtonElement>(null);
    const agentPickerRef = React.useRef<HTMLDivElement>(null);

    const [position, setPosition] = React.useState<DragPosition | null>(null);
    const [isDragging, setIsDragging] = React.useState(false);
    const dragStartRef = React.useRef<{
      clientX: number;
      clientY: number;
      position: DragPosition;
    } | null>(null);

    React.useImperativeHandle(ref, () => innerRef.current!);

    const rawCtx = React.useMemo(
      () => resolvePageContext(contextMap, pageContext),
      [contextMap, pageContext],
    );
    const ctx = React.useMemo(() => ({
      page: rawCtx.page || 'Workbench',
      greeting: rawCtx.greeting || '',
      actions: rawCtx.actions || [],
    }), [rawCtx]);

    // 初始化问候消息
    React.useEffect(() => {
      if (!initializedRef.current) {
        setMessages([{ role: "ai", text: ctx.greeting }]);
        initializedRef.current = true;
      }
    }, [ctx.greeting]);

    // 自动滚动到底部
    React.useEffect(() => {
      if (scrollRef.current) {
        scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
      }
    }, [messages]);

    // 打开时聚焦输入框
    React.useEffect(() => {
      if (isOpen && inputRef.current) {
        setTimeout(() => inputRef.current?.focus(), 100);
      }
    }, [isOpen]);

    const send = React.useCallback(
      async (text?: string) => {
        const msg = (text || input).trim();
        if (!msg || loading) return;

        setMessages((prev) => [...prev, { role: "user", text: msg }]);
        setInput("");
        setLoading(true);

        try {
          const reply = await onSend(msg, ctx);
          if (typeof reply === "string") {
            setMessages((prev) => [...prev, { role: "ai", text: reply }]);
          } else {
            setMessages((prev) => [
              ...prev,
              {
                role: "ai",
                text: reply.text,
                agent_code: reply.agent_code,
                structured: reply.structured,
                requires_input: reply.requires_input,
                suggested_actions: reply.suggested_actions,
              },
            ]);
          }
        } catch {
          setMessages((prev) => [
            ...prev,
            { role: "ai", text: t("components.ai.requestFailed") },
          ]);
        } finally {
          setLoading(false);
        }
      },
      [input, loading, onSend, ctx],
    );

    const handleKeyDown = React.useCallback(
      (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          send();
        }
      },
      [send],
    );

    const placeholder = resolvedPlaceholderTemplate.replace("{page}", ctx.page);

    // ── 拖拽逻辑 ──
    const handlePointerDown = React.useCallback(
      (e: React.PointerEvent<HTMLButtonElement>) => {
        if (!innerRef.current) return;
        (e.target as HTMLElement).setPointerCapture(e.pointerId);

        const rect = innerRef.current.getBoundingClientRect();
        const startPos: DragPosition = position ?? {
          ...(right
            ? { right: window.innerWidth - rect.right }
            : { left: rect.left }),
          bottom: window.innerHeight - rect.bottom,
        };

        dragStartRef.current = {
          clientX: e.clientX,
          clientY: e.clientY,
          position: startPos,
        };
        setIsDragging(true);
      },
      [position, right],
    );

    const handlePointerMove = React.useCallback(
      (e: React.PointerEvent<HTMLButtonElement>) => {
        if (!isDragging || !dragStartRef.current) return;

        const { clientX, clientY, position: startPos } = dragStartRef.current;
        const dx = e.clientX - clientX;
        const dy = e.clientY - clientY;

        const buttonSize = 40;
        const maxLeft = window.innerWidth - buttonSize;
        const maxBottom = window.innerHeight - buttonSize;

        let next: DragPosition;
        if (startPos.right !== undefined) {
          const nextRight = Math.max(0, Math.min(maxLeft, startPos.right - dx));
          next = { right: nextRight, bottom: Math.max(0, Math.min(maxBottom, startPos.bottom - dy)) };
        } else {
          const nextLeft = Math.max(0, Math.min(maxLeft, (startPos.left ?? 24) + dx));
          next = { left: nextLeft, bottom: Math.max(0, Math.min(maxBottom, startPos.bottom - dy)) };
        }

        setPosition(next);
      },
      [isDragging],
    );

    const handlePointerUp = React.useCallback(
      (e: React.PointerEvent<HTMLButtonElement>) => {
        if (!dragStartRef.current) return;

        const dx = e.clientX - dragStartRef.current.clientX;
        const dy = e.clientY - dragStartRef.current.clientY;
        const distance = Math.sqrt(dx * dx + dy * dy);

        setIsDragging(false);
        dragStartRef.current = null;

        if (distance < 5) {
          setIsOpen(!isOpen);
        }
      },
      [],
    );

    React.useEffect(() => {
      if (!isDragging) return;
      const prevent = (e: Event) => e.preventDefault();
      document.addEventListener("selectstart", prevent);
      return () => document.removeEventListener("selectstart", prevent);
    }, [isDragging]);

    const hasCustomPosition = position !== null;
    const horizontalClass = right ? right : left;

    const buttonStyle: React.CSSProperties = {
      width: 40,
      height: 40,
      ...(hasCustomPosition
        ? {
            ...(position.right !== undefined
              ? { right: position.right }
              : { left: position.left ?? 24 }),
            bottom: position.bottom,
          }
        : {}),
    };

    if (docked && !isOpen) return null;

    return (
      <>
        {/* 浮动触发按钮 */}
        {!docked && (
          <button
            ref={innerRef}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
            className={cn(
              "fixed z-50 flex h-10 w-10 items-center justify-center rounded-full aspect-square shrink-0",
              "bg-primary text-primary-foreground shadow-lg shadow-primary/30",
              "transition-transform transition-colors duration-300 hover:scale-110",
              "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
              "cursor-grab active:cursor-grabbing",
              isOpen ? "rotate-90" : !hasCustomPosition && "animate-[ai-float_3s_ease-in-out_infinite]",
              !hasCustomPosition && bottom,
              !hasCustomPosition && horizontalClass,
              className,
            )}
            style={buttonStyle}
            aria-label={isOpen ? t("components.ai.closeAssistant") : t("components.ai.openAssistant")}
          >
            <style>{`
              @keyframes ai-float {
                0%, 100% { transform: translateY(0px); }
                50% { transform: translateY(-6px); }
              }
            `}</style>
            {isOpen ? (
              <X className="h-4 w-4" />
            ) : (
              <Bot className="h-4 w-4" />
            )}
          </button>
        )}

        {/* 展开面板 */}
        {isOpen && (
          <div
            className={cn(
              docked
                ? "flex h-full w-full flex-col bg-card"
                : "fixed inset-y-0 right-0 z-40 flex w-full flex-col bg-card shadow-2xl sm:w-96",
              "border-l border-border",
              !docked && "animate-[ai-slide-in_0.2s_ease-out]",
              panelClassName,
            )}
          >
            <style>{`
              @keyframes ai-slide-in {
                from { transform: translateX(100%); }
                to { transform: translateX(0); }
              }
            `}</style>

            {/* Agent 选择器 */}

            {agents.length > 0 && (
              <div className="border-b px-4 py-2">
                <div className="relative" ref={agentPickerRef}>
                  <button
                    onClick={() => setShowAgentPicker((v) => !v)}
                    className={cn(
                      "flex w-full items-center justify-between rounded-lg border px-3 py-1.5 text-xs",
                      "text-muted-foreground hover:border-primary/30 hover:bg-primary/5",
                      showAgentPicker && "border-primary/30 bg-primary/5",
                    )}
                  >
                    <span className="flex items-center gap-2">
                      <Bot className="h-3 w-3" />
                      {agentCode
                        ? agents.find((a) => a.code === agentCode)?.name || t("ai.agent.select")
                        : t("ai.agent.select")}
                    </span>
                    <span className="text-xs">{showAgentPicker ? "▲" : "▼"}</span>
                  </button>
                  {showAgentPicker && (
                    <div className="absolute z-50 mt-1 w-full rounded-lg border bg-card shadow-lg">
                      {agents.map((agent) => (
                        <button
                          key={agent.code}
                          onClick={() => {
                            onAgentChange?.(agent.code);
                            setShowAgentPicker(false);
                          }}
                          className={cn(
                            "flex w-full items-center gap-2 px-3 py-2 text-left text-xs",
                            "hover:bg-primary/5",
                            agentCode === agent.code && "bg-primary/10 font-medium text-primary",
                          )}
                        >
                          <span
                            className="flex h-4 w-4 shrink-0 items-center justify-center rounded-full text-xs font-bold text-primary-foreground"
                            style={{ backgroundColor: agent.color || "#6366f1" }}
                          >
                            {agent.name.charAt(0)}
                          </span>
                          <span className="flex-1">{agent.name}</span>
                          {agent.category && (
                            <span className="text-xs text-muted-foreground">{agent.category}</span>
                          )}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* 快捷操作 */}
            {ctx.actions.length > 0 && (
              <div className="px-4 pb-1 pt-3">
                <p className="mb-2 text-xs uppercase tracking-wider text-muted-foreground">
                  {t("components.ai.quickActions")}
                </p>
                <div className="flex flex-wrap gap-2">
                  {ctx.actions.map((action) => (
                    <button
                      key={action}
                      onClick={() => send(action)}
                      disabled={loading}
                      className={cn(
                        "rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors",
                        "text-muted-foreground hover:border-primary/30 hover:bg-primary/5 hover:text-primary",
                        "disabled:cursor-not-allowed disabled:opacity-50",
                      )}
                    >
                      {action}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* 消息区 */}
            <div
              ref={scrollRef}
              className="flex-1 space-y-4 overflow-y-auto p-4"
            >
              {messages.map((msg, i) => (
                <div
                  key={i}
                  className={cn(
                    "flex",
                    msg.role === "user" ? "justify-end" : "justify-start",
                  )}
                >
                  <div
                    className={cn(
                      "rounded-2xl px-4 py-2.5 text-sm leading-relaxed",
                      msg.role === "user"
                        ? "rounded-br-md bg-primary text-primary-foreground"
                        : "rounded-bl-md bg-muted text-foreground",
                    )}
                    style={{ maxWidth: '80%' }}
                  >
                    {msg.text}

                    {/* 结构化数据展示 */}
                    {msg.structured && (
                      <details className="mt-2 rounded-lg border border-border/50 bg-background/50">
                        <summary className="cursor-pointer px-2 py-1 text-xs text-muted-foreground">
                          {t("components.ai.structuredData")}
                        </summary>
                        <pre className="max-h-32 overflow-auto p-2 text-xs">
                          {JSON.stringify(msg.structured, null, 2)}
                        </pre>
                      </details>
                    )}

                    {/* Action 按钮 */}
                    {msg.requires_input && msg.suggested_actions && msg.suggested_actions.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1.5">
                        {msg.suggested_actions.map((action) => (
                          <button
                            key={action}
                            onClick={() => send(action)}
                            className={cn(
                              "rounded-md border px-2 py-1 text-xs font-medium transition-colors",
                              "border-primary/20 bg-primary/5 text-primary hover:bg-primary/10",
                            )}
                          >
                            {action}
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              ))}
              {loading && (
                <div className="flex justify-start">
                  <div className="flex items-center gap-2 rounded-2xl rounded-bl-md bg-muted px-4 py-2.5">
                    <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                    <span className="text-sm text-muted-foreground">
                      {t("components.ai.thinking")}
                    </span>
                  </div>
                </div>
              )}
            </div>

            {/* 输入区 */}
            <div className="border-t p-4">
              <div className="flex items-center gap-2">
                <input
                  ref={inputRef}
                  type="text"
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder={placeholder}
                  disabled={loading}
                  className={cn(
                    "min-w-0 flex-1 rounded-xl border bg-muted px-4 py-2.5 text-sm",
                    "transition-colors",
                    "focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/30",
                    "disabled:cursor-not-allowed disabled:opacity-50",
                  )}
                />
                <button
                  onClick={() => send()}
                  disabled={loading || !input.trim()}
                  className={cn(
                    "flex h-10 w-10 items-center justify-center rounded-xl bg-primary text-primary-foreground",
                    "transition-colors hover:bg-primary/90",
                    "disabled:cursor-not-allowed disabled:opacity-50",
                  )}
                  aria-label={t("components.ai.send")}
                >
                  <Send className="h-4 w-4" />
                </button>
              </div>
              <p className="mt-2 text-center text-xs text-muted-foreground">
                {t("components.ai.disclaimer")}
              </p>
            </div>
          </div>
        )}
      </>
    );
  },
);

AIChatPanel.displayName = "AIChatPanel";
