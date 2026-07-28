import { createApiClient, type ApiClient } from "./client.js";
import { getApiBaseURL } from "./runtime.js";

export interface AIChatConfig {
  /** Meta 聊天 API 的 baseURL，默认 "/api" */
  baseURL?: string;
  /**
   * 首次创建会话时使用的标题，默认 "AI 助手对话"。
   * 后续消息沿用同一会话。
   */
  sessionTitle?: string;
  /**
   * 创建会话时指定的默认 Agent
   */
  defaultAgent?: string;
  /** 自定义会话存储（默认使用 PersistentSessionStore） */
  sessionStore?: ChatSessionStore;
}

export interface AIChatMessage {
  content: string;
  agent_code: string;
  structured?: Record<string, unknown>;
  requires_input?: boolean;
  suggested_actions?: string[];
}

export interface AIChatService {
  /**
   * 发送用户消息并获取 AI 回复。
   * 自动管理会话生命周期（首次调用创建会话，后续复用）。
   */
  sendMessage: (message: string, context?: AIChatContext) => Promise<AIChatMessage>;
  /** 清除当前会话，下次调用将创建新会话 */
  resetSession: () => void;
  /** 获取可用 Agent 列表 */
  getAgents: () => Promise<AgentInfo[]>;
  /** 切换当前会话的 Agent */
  switchAgent: (agentCode: string) => Promise<void>;
}

export interface AIChatContext {
  /** 页面上下文 */
  pageContext?: {
    module?: string;
    page?: string;
    currentData?: Record<string, unknown>;
    recentActions?: string[];
    availableOperations?: string[];
    extraContext?: string;
    suggestedAgent?: string;
    intentHints?: string[];
  };
}

export interface AgentInfo {
  code: string;
  name: string;
  description: string;
  capabilities: string[];
  user_selectable: boolean;
  sort_order: number;
  /** Lucide 图标名 */
  icon: string;
  /** UI 颜色 */
  color: string;
  /** 分类 */
  category: string;
}

interface ChatMessageData {
  id: number;
  role: string;
  content: string;
  created_at: string;
  agent_code: string;
  structured?: Record<string, unknown>;
  requires_input?: boolean;
  suggested_actions?: string[];
}

export interface ChatSessionStore {
  /** 获取当前存储的会话 ID（可能为 null） */
  getSessionId(): string | null;
  /** 确保有活跃会话（如果不存在则创建） */
  ensureSession(context?: AIChatContext): Promise<string>;
  /** 添加用户消息 */
  addMessage(sessionId: string, content: string): Promise<void>;
  /** 生成 AI 回复 */
  generateResponse(sessionId: string): Promise<ChatMessageData>;
  /** 清除会话 */
  resetSession(): void;
}

class PersistentSessionStore implements ChatSessionStore {
  private client: ApiClient;
  private sessionTitle: string;
  private defaultAgent?: string;
  private storageKey = "alioth_ai_chat_session_id";

  constructor(client: ApiClient, sessionTitle: string, defaultAgent?: string) {
    this.client = client;
    this.sessionTitle = sessionTitle;
    this.defaultAgent = defaultAgent;
  }

  getSessionId(): string | null {
    try {
      return localStorage.getItem(this.storageKey);
    } catch {
      return null;
    }
  }

  async ensureSession(context?: AIChatContext): Promise<string> {
    const stored = this.getSessionId();
    if (stored) return stored;

    const payload: Record<string, unknown> = { title: this.sessionTitle };
    if (this.defaultAgent) {
      payload.agent_code = this.defaultAgent;
    }
    if (context?.pageContext) {
      payload.context = context.pageContext;
    }

    const response = await this.client.post<{ success: boolean; data?: { id: number } }>(
      "/chat-sessions",
      payload,
    );
    const id = String(response?.data?.id ?? "");
    if (id) {
      try {
        localStorage.setItem(this.storageKey, id);
      } catch {
        // ignore
      }
    }
    return id;
  }

  async addMessage(sessionId: string, content: string): Promise<void> {
    await this.client.post(`/chat-sessions/${sessionId}/messages`, {
      role: "user",
      content,
    });
  }

  async generateResponse(sessionId: string): Promise<ChatMessageData> {
    const response = await this.client.post<{ success: boolean; data?: ChatMessageData }>(
      `/chat-sessions/${sessionId}/generate-response`,
    );
    return (
      response?.data ?? {
        id: 0,
        role: "assistant",
        content: "抱歉，无法获取 AI 回复。",
        created_at: new Date().toISOString(),
        agent_code: "general",
      }
    );
  }

  resetSession(): void {
    try {
      localStorage.removeItem(this.storageKey);
    } catch {
      // ignore
    }
  }
}

export class InMemorySessionStore implements ChatSessionStore {
  private sessionId: string | null = null;
  private messageLog: Array<{ sessionId: string; content: string }> = [];
  private responses: Map<string, ChatMessageData> = new Map();
  private createCallCount = 0;
  private shouldFailOnFirstAccess = false;
  private accessCount = 0;

  getSessionId(): string | null {
    return this.sessionId;
  }

  async ensureSession(_context?: AIChatContext): Promise<string> {
    this.createCallCount++;
    this.accessCount++;
    if (this.shouldFailOnFirstAccess && this.accessCount === 1) {
      throw new Error("Session invalid");
    }
    if (!this.sessionId) {
      this.sessionId = `test-session-${this.createCallCount}`;
    }
    return this.sessionId;
  }

  async addMessage(sessionId: string, content: string): Promise<void> {
    this.messageLog.push({ sessionId, content });
  }

  async generateResponse(sessionId: string): Promise<ChatMessageData> {
    return (
      this.responses.get(sessionId) ?? {
        id: 1,
        role: "assistant",
        content: "Test response",
        created_at: new Date().toISOString(),
        agent_code: "general",
      }
    );
  }

  resetSession(): void {
    this.sessionId = null;
    this.accessCount = 0;
  }

  // 测试辅助方法
  simulateInvalidSessionOnFirstAccess(): void {
    this.shouldFailOnFirstAccess = true;
  }

  setResponse(sessionId: string, response: ChatMessageData): void {
    this.responses.set(sessionId, response);
  }

  get createCount(): number {
    return this.createCallCount;
  }

  get messages(): Array<{ sessionId: string; content: string }> {
    return this.messageLog;
  }
}

export function createAIChatService(config: AIChatConfig = {}): AIChatService {
  const baseURL = config.baseURL || getApiBaseURL();
  const sessionTitle = config.sessionTitle || "AI 助手对话";
  const defaultAgent = config.defaultAgent;
  const client: ApiClient = createApiClient({ baseURL });

  const store: ChatSessionStore =
    config.sessionStore || new PersistentSessionStore(client, sessionTitle, defaultAgent);

  async function getAgents(): Promise<AgentInfo[]> {
    const response = await client.get<{ success: boolean; data?: AgentInfo[] }>(
      "/chat-sessions/agents",
    );
    return response?.data ?? [];
  }

  async function switchAgent(agentCode: string): Promise<void> {
    const sessionId = store.getSessionId();
    if (!sessionId) {
      throw new Error("No active session");
    }
    await client.post(`/chat-sessions/${sessionId}/switch-agent`, {
      agent_code: agentCode,
    });
  }

  return {
    sendMessage: async (message: string, context?: AIChatContext): Promise<AIChatMessage> => {
      let sessionId = await store.ensureSession(context);

      await store.addMessage(sessionId, message);

      try {
        const data = await store.generateResponse(sessionId);
        return {
          content: data.content,
          agent_code: data.agent_code,
          structured: data.structured,
          requires_input: data.requires_input,
          suggested_actions: data.suggested_actions,
        };
      } catch {
        // 如果会话已失效，重新创建并重试一次
        store.resetSession();
        sessionId = await store.ensureSession(context);
        await store.addMessage(sessionId, message);
        const data = await store.generateResponse(sessionId);
        return {
          content: data.content,
          agent_code: data.agent_code,
          structured: data.structured,
          requires_input: data.requires_input,
          suggested_actions: data.suggested_actions,
        };
      }
    },

    resetSession: () => store.resetSession(),

    getAgents,
    switchAgent,
  };
}
