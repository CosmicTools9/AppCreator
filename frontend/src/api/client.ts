/**
 * API client — wraps fetch with JWT auth header.
 * Tokens provided by stores/auth.ts via the getToken callback.
 */

export type ApiOptions = {
  /** JWT token (null = no auth). */
  token: string | null;
};

const BASE = '/api/creator';

// ── Token / Refresh helpers ───────────────────────────

/** Get the stored refresh token (used by 401 interceptor). */
function getRefreshToken(): string | null {
  return localStorage.getItem('sso_refresh_token');
}

/** Store refresh token after a refresh or login. */
export function setRefreshToken(token: string | null) {
  if (token) localStorage.setItem('sso_refresh_token', token);
  else localStorage.removeItem('sso_refresh_token');
}

/** Store access token after a refresh or login (exported for auth.ts). */
export function setAccessToken(token: string | null) {
  if (token) localStorage.setItem('sso_token', token);
  else localStorage.removeItem('sso_token');
}

/**
 * Attempt to refresh the access token using the stored refresh_token.
 * Returns the new token string on success, null on failure.
 */
async function tryRefreshToken(): Promise<string | null> {
  const rt = getRefreshToken();
  if (!rt) return null;
  try {
    const res = await fetch(`${BASE}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: rt }),
    });
    if (!res.ok) {
      setRefreshToken(null);
      return null;
    }
    const data = await res.json();
    const token = data.token ?? data.data?.token;
    const newRt = data.refresh_token ?? data.data?.refresh_token;
    if (token) setAccessToken(token);
    if (newRt) setRefreshToken(newRt);
    return token ?? null;
  } catch {
    return null;
  }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(path: string, options: RequestInit & ApiOptions): Promise<T> {
  const doFetch = (token: string | null): Promise<Response> => {
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (token) headers['Authorization'] = `Bearer ${token}`;
    return fetch(`${BASE}${path}`, { ...options, headers });
  };

  let res = await doFetch(options.token ?? null);

  // 401 auto-refresh: try once if refresh_token available
  if (res.status === 401) {
    const newToken = await tryRefreshToken();
    if (newToken) {
      res = await doFetch(newToken);
    }
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    const b = body as { error?: string; message?: string };
    throw new ApiError(res.status, b.error ?? b.message ?? res.statusText);
  }
  const json = await res.json();
  if (json && typeof json === 'object' && 'success' in json && 'data' in json) {
    return json.data as T;
  }
  return json as T;
}

export interface ChatSession {
  id: number;
  title: string | null;
  app_instance_id: number | null;
  namespace: string;
  status: string;
  created_at: string;
  updated_at: string;
  messages?: ChatMessage[];
}

export interface ChatMessage {
  id: number;
  session_id: number;
  role: 'user' | 'assistant';
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

export interface AppInfo {
  code: string;
  app_json: Record<string, unknown>;
}

export interface RefreshResponse {
  token: string;
  refresh_token: string;
  user: { id: number; username: string; namespace: string };
}

export interface ProgressResponse {
  status: string;
  progress: number;
  state_before: string;
  state_after: string;
  is_terminal: boolean;
}

export interface ResetStateRequest {
  target_state?: string;
}

export const api = {
  // ── Chat Sessions ───────────────────────────────────
  createSession: (
    body: { title?: string; app_instance_id?: number | null; namespace: string },
    opts: ApiOptions,
  ) =>
    request<ChatSession>('/sessions', {
      method: 'POST',
      body: JSON.stringify(body),
      ...opts,
    }),

  createApp: (body: { name: string; description: string }, opts: ApiOptions) =>
    request<CreateAppResponse>('/apps', {
      method: 'POST',
      body: JSON.stringify(body),
      ...opts,
    }),

  getSession: (id: number, opts: ApiOptions) =>
    request<ChatSession>(`/sessions/${id}`, { method: 'GET', ...opts }),

  listSessions: (namespace: string | null, opts: ApiOptions) =>
    request<{ sessions: ChatSession[] }>(
      `/sessions${namespace ? `?namespace=${encodeURIComponent(namespace)}` : ''}`,
      { method: 'GET', ...opts },
    ),

  addMessage: (
    id: number,
    body: { content: string; role?: 'user' | 'assistant' },
    opts: ApiOptions,
  ) =>
    request<ChatMessage>(`/sessions/${id}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content: body.content, role: body.role ?? 'user' }),
      ...opts,
    }),

  generateResponse: (id: number, opts: ApiOptions) =>
    request<StepResponse>(`/sessions/${id}/generate-response`, {
      method: 'POST',
      ...opts,
    }),

  /** Fetch prototype.html as raw text (auth header attached; not JSON). */
  fetchPrototype: async (id: number, opts: ApiOptions): Promise<string> => {
    const headers: Record<string, string> = {};
    if (opts.token) {
      headers['Authorization'] = `Bearer ${opts.token}`;
    }
    const res = await fetch(`${BASE}/sessions/${id}/prototype`, { headers });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      const b = body as { error?: string; message?: string };
      throw new ApiError(res.status, b.error ?? b.message ?? res.statusText);
    }
    return res.text();
  },

  interrupt: (id: number, opts: ApiOptions) =>
    request<{ status: string }>(`/sessions/${id}/interrupt`, {
      method: 'POST',
      ...opts,
    }),

  // ── Auth ──────────────────────────────────────────────
  refreshToken: (refreshToken: string, opts: ApiOptions) =>
    request<RefreshResponse>('/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken }),
      ...opts,
    }),

  // ── Session lifecycle ────────────────────────────────
  resume: (id: number, opts: ApiOptions) =>
    request<StepResponse>(`/sessions/${id}/resume`, {
      method: 'POST',
      ...opts,
    }),

  resetState: (id: number, body: ResetStateRequest, opts: ApiOptions) =>
    request<{ status: string }>(`/sessions/${id}/reset-state`, {
      method: 'POST',
      body: JSON.stringify(body),
      ...opts,
    }),

  progress: (id: number, opts: ApiOptions) =>
    request<ProgressResponse>(`/sessions/${id}/progress`, {
      method: 'GET',
      ...opts,
    }),

  // ── Apps ─────────────────────────────────────────────
  listApps: (opts: ApiOptions) => request<{ apps: AppInfo[] }>('/apps', { method: 'GET', ...opts }),

  getApp: (code: string, opts: ApiOptions) =>
    request<{ app: Record<string, unknown> }>(`/apps/${encodeURIComponent(code)}`, {
      method: 'GET',
      ...opts,
    }),

  deleteApp: (code: string, opts: ApiOptions) =>
    request<{ status: string }>(`/apps/${encodeURIComponent(code)}`, {
      method: 'DELETE',
      ...opts,
    }),
};
