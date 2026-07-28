/**
 * PageContextModule — 深度封装页面上下文收集、验证、渲染
 *
 * 接口：
 *   register(ctx: AIPageContext): void
 *   snapshot(): ContextEnvelope
 *   renderForAgent(agentCode: string): RenderedContext
 *   clear(): void
 *
 * 隐藏：
 *   - 字段验证与裁剪
 *   - 版本化序列化
 *   - 过期检测
 *   - Agent 特定模板渲染
 */

// ═══════════════════════════════════════════
// Types
// ═══════════════════════════════════════════

/**
 * 页面注册的 AI 上下文
 */
export interface AIPageContext {
  /**
   * 所属模块标识（如 "product", "orders", "inventory"）
   */
  module?: string;

  /**
   * 当前页面名称（如 "产品列表", "订单详情", "库存盘点"）
   */
  page?: string;

  /**
   * 当前页面的数据快照（简明摘要，不超过 500 字符）
   * 示例：{ filter: "品类=电子", resultCount: 156 }
   */
  currentData?: Record<string, unknown>;

  /**
   * 用户最近的操作记录（最近 3-5 条）
   * 示例：["查看了产品 TH-PRO-2024", "修改了库存阈值", "审批了订单 #2024-089"]
   */
  recentActions?: string[];

  /**
   * 当前页面可用的操作列表
   * 示例：["新建产品", "批量导入", "导出报表", "筛选"]
   */
  availableOperations?: string[];

  /**
   * 额外的自由格式上下文（最长 1000 字符）
   * 可用于传递无法用上述字段表达的上下文
   */
  extraContext?: string;

  /**
   * 建议默认使用的 Agent 编码
   * 如 "data_analysis", "form_filling", "flow_design" 等
   */
  suggestedAgent?: string;

  /**
   * 意图提示词列表，辅助 Agent 路由
   * 如 ["分析", "统计", "报表"] 等
   */
  intentHints?: string[];
}

/**
 * 全局 AI 上下文状态
 */
export interface AIContextState {
  /** 页面注册的上下文 */
  pageContext: AIPageContext | null;
  /** 上下文注册时间戳 */
  registeredAt: number | null;
}

/**
 * 版本化上下文信封
 */
export interface ContextEnvelope {
  version: "2026-05-30";
  timestamp: number;
  expiresAt: number;
  page: string;
  module: string;
  raw: AIPageContext;
  rendered: Map<string, RenderedContext>;
}

/**
 * 针对特定 Agent 渲染后的上下文
 */
export interface RenderedContext {
  agentCode: string;
  text: string;           // 渲染后的 system prompt
  structured?: unknown;   // 结构化数据（如表单 schema）
  metadata: {
    templateVersion: string;
    fieldsIncluded: string[];
  };
}

// ═══════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════

const DEFAULT_TTL_MS = 5 * 60 * 1000; // 5 分钟过期
const MAX_EXTRA_CONTEXT_LENGTH = 1000;

// ═══════════════════════════════════════════
// PageContextModule
// ═══════════════════════════════════════════

export class PageContextModule {
  private state: AIContextState = { pageContext: null, registeredAt: null };
  private readonly TTL_MS: number;

  constructor(ttlMs = DEFAULT_TTL_MS) {
    this.TTL_MS = ttlMs;
  }

  register(ctx: AIPageContext): void {
    this.validateContext(ctx);

    const validated: AIPageContext = {
      ...ctx,
      extraContext: ctx.extraContext
        ? this.truncateField(ctx.extraContext, MAX_EXTRA_CONTEXT_LENGTH)
        : undefined,
    };

    this.state = {
      pageContext: validated,
      registeredAt: Date.now(),
    };
  }

  snapshot(): ContextEnvelope {
    const { pageContext, registeredAt } = this.state;
    const now = Date.now();

    if (!pageContext || !registeredAt || now > registeredAt + this.TTL_MS) {
      return {
        version: "2026-05-30",
        timestamp: now,
        expiresAt: now,
        page: "",
        module: "",
        raw: {},
        rendered: new Map(),
      };
    }

    const rendered = new Map<string, RenderedContext>();
    // 预渲染通用模板
    rendered.set("general", this.renderForAgentInternal("general", pageContext));
    // 预渲染建议的 Agent 模板
    if (pageContext.suggestedAgent) {
      rendered.set(
        pageContext.suggestedAgent,
        this.renderForAgentInternal(pageContext.suggestedAgent, pageContext),
      );
    }

    return {
      version: "2026-05-30",
      timestamp: now,
      expiresAt: registeredAt + this.TTL_MS,
      page: pageContext.page ?? "",
      module: pageContext.module ?? "",
      raw: pageContext,
      rendered,
    };
  }

  renderForAgent(agentCode: string): RenderedContext {
    const { pageContext } = this.state;
    if (!pageContext) {
      return {
        agentCode,
        text: "",
        metadata: { templateVersion: "empty", fieldsIncluded: [] },
      };
    }
    return this.renderForAgentInternal(agentCode, pageContext);
  }

  clear(): void {
    this.state = { pageContext: null, registeredAt: null };
  }

  // ── Private ──

  private validateContext(ctx: AIPageContext): void {
    if (ctx.extraContext && typeof ctx.extraContext !== "string") {
      console.warn(
        "[PageContextModule] extraContext should be a string, got",
        typeof ctx.extraContext,
      );
    }
    if (ctx.currentData && typeof ctx.currentData !== "object") {
      console.warn(
        "[PageContextModule] currentData should be an object, got",
        typeof ctx.currentData,
      );
    }
  }

  private truncateField(value: string, maxLen: number): string {
    if (!value || value.length <= maxLen) return value;
    return value.slice(0, maxLen) + "…";
  }

  private renderForAgentInternal(
    agentCode: string,
    ctx: AIPageContext,
  ): RenderedContext {
    const fieldsIncluded: string[] = [];
    const parts: string[] = [];

    parts.push("## 当前页面上下文");

    if (ctx.module) {
      parts.push(`- 所属模块：${ctx.module}`);
      fieldsIncluded.push("module");
    }
    if (ctx.page) {
      parts.push(`- 当前页面：${ctx.page}`);
      fieldsIncluded.push("page");
    }

    if (ctx.currentData && Object.keys(ctx.currentData).length > 0) {
      parts.push(`- 当前数据：${JSON.stringify(ctx.currentData)}`);
      fieldsIncluded.push("currentData");
    }

    if (ctx.recentActions && ctx.recentActions.length > 0) {
      parts.push(`- 最近操作：${ctx.recentActions.join("、")}`);
      fieldsIncluded.push("recentActions");
    }

    if (ctx.availableOperations && ctx.availableOperations.length > 0) {
      parts.push(`- 可用操作：${ctx.availableOperations.join("、")}`);
      fieldsIncluded.push("availableOperations");
    }

    if (ctx.extraContext) {
      parts.push(`- 补充信息：${ctx.extraContext}`);
      fieldsIncluded.push("extraContext");
    }

    let structured: unknown | undefined;
    let templateVersion = "general-v1";

    if (agentCode === "form_filling") {
      templateVersion = "form_filling-v1";
      structured = {
        hint: "当前页面可能包含表单操作，AI 可协助填写或校验表单数据。",
        availableOperations: ctx.availableOperations ?? [],
      };
    } else if (agentCode === "data_analysis") {
      templateVersion = "data_analysis-v1";
      structured = {
        hint: "当前页面可能包含数据展示，AI 可协助分析、统计或生成报表。",
        currentData: ctx.currentData ?? {},
      };
    }

    return {
      agentCode,
      text: parts.join("\n"),
      structured,
      metadata: {
        templateVersion,
        fieldsIncluded,
      },
    };
  }
}

/** 全局单例实例 */
export const pageContextModule = new PageContextModule();
