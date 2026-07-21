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
    ...(options.headers as Record<string, string>),
  };
  if (options.token) {
    headers["Authorization"] = `Bearer ${options.token}`;
  }
  const res = await fetch(`${BASE}${path}`, { ...options, headers });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new ApiError(res.status, body.error ?? body.message ?? res.statusText);
  }
  return res.json();
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

// ── Projects ──────────────────────────────────────────

export interface Project {
  id: number;
  name: string;
  status: string;
  created_at: string;
}

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

export const api = {
  listProjects: (opts: ApiOptions) =>
    request<{ projects: Project[]; total: number }>("/projects", {
      method: "GET",
      ...opts,
    }),

  getProject: (id: number, opts: ApiOptions) =>
    request<Project>(`/projects/${id}`, { method: "GET", ...opts }),

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

  getSession: (id: number, opts: ApiOptions) =>
    request<ChatSession>(`/sessions/${id}`, { method: "GET", ...opts }),

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
};
