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

export const api = {
  listProjects: (opts: ApiOptions) =>
    request<{ projects: Project[]; total: number }>("/projects", {
      method: "GET",
      ...opts,
    }),

  getProject: (id: number, opts: ApiOptions) =>
    request<Project>(`/projects/${id}`, { method: "GET", ...opts }),
};
