/**
 * API client — wraps fetch with JWT auth header.
 * Tokens provided by stores/auth.ts via the getToken callback.
 */

export type ApiOptions = {
  /** JWT token (null = no auth). */
  token: string | null;
};

const BASE = "/api/creator";

async function request<T>(
  path: string,
  options: RequestInit & ApiOptions
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (options.token) {
    headers["Authorization"] = `Bearer ${options.token}`;
  }
  const res = await fetch(`${BASE}${path}`, { ...options, headers });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    const b = body as { error?: string; message?: string };
    throw new ApiError(res.status, b.error ?? b.message ?? res.statusText);
  }
  const json = await res.json();
  // 解包后端 ApiResponse 格式 `{ success: boolean, data: T }`（与 Meta 前端惯例一致）
  if (json && typeof json === "object" && "success" in json && "data" in json) {
    return json.data as T;
  }
  return json as T;
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

// ── Chat ──────────────────────────────────────────────

export interface ChatSession {
  id: number;
  title: string;
  app_instance_id: number | null;
  namespace: string;
  status: string;
  created_at: string;
  updated_at: string;
  messages: ChatMessage[];
}

export interface ChatMessage {
  id: number;
  session_id: number;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

export interface StepResponse {
  state_before: string;
  state_after: string;
  is_terminal: boolean;
  progress_percent: number;
  message: string;
}

export interface CreateAppResponse {
  session: ChatSession;
  app_name: string;
}

export const api = {
  // ── Chat Sessions ───────────────────────────────────
  createSession: (
    body: { title?: string; app_instance_id?: number | null; namespace: string },
    opts: ApiOptions
  ) =>
    request<ChatSession>("/sessions", {
      method: "POST",
      body: JSON.stringify(body),
      ...opts,
    }),

  createApp: (
    body: { name: string; description: string },
    opts: ApiOptions
  ) =>
    request<CreateAppResponse>("/apps", {
      method: "POST",
      body: JSON.stringify(body),
      ...opts,
    }),

  getSession: (id: number, opts: ApiOptions) =>
    request<ChatSession>(`/sessions/${id}`, { method: "GET", ...opts }),

  listSessions: (namespace: string | null, opts: ApiOptions) =>
    request<{ sessions: ChatSession[] }>(
      `/sessions${namespace ? `?namespace=${encodeURIComponent(namespace)}` : ""}`,
      { method: "GET", ...opts }
    ),

  addMessage: (
    id: number,
    body: { content: string; role?: "user" | "assistant" },
    opts: ApiOptions
  ) =>
    request<ChatMessage>(`/sessions/${id}/messages`, {
      method: "POST",
      body: JSON.stringify({ content: body.content, role: body.role ?? "user" }),
      ...opts,
    }),

  generateResponse: (id: number, opts: ApiOptions) =>
    request<StepResponse>(`/sessions/${id}/generate-response`, {
      method: "POST",
      ...opts,
    }),

  /** Fetch prototype.html as raw text (auth header attached; not JSON). */
  fetchPrototype: async (id: number, opts: ApiOptions): Promise<string> => {
    const headers: Record<string, string> = {};
    if (opts.token) {
      headers["Authorization"] = `Bearer ${opts.token}`;
    }
    const res = await fetch(`${BASE}/sessions/${id}/prototype`, { headers });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      const b = body as { error?: string; message?: string };
      throw new ApiError(res.status, b.error ?? b.message ?? res.statusText);
    }
    return res.text();
  },
};
