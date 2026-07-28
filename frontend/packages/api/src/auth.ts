export interface TokenPair {
  accessToken: string;
  refreshToken?: string;
}

// Token管理 (migrated to httpOnly cookies)
export const tokenManager = {
  getAccessToken(): string | null {
    return null; // token now lives in httpOnly cookie
  },

  setAccessToken(_token: string): void {
    // no-op: token is delivered via httpOnly cookie
  },

  getRefreshToken(): string | null {
    return null; // refresh token now lives in httpOnly cookie
  },

  setRefreshToken(_token: string): void {
    // no-op: refresh token is delivered via httpOnly cookie
  },

  setTokens(_tokens: TokenPair): void {
    // no-op: tokens are delivered via httpOnly cookies
  },

  clearTokens(): void {
    // no-op: cookies are cleared by backend logout endpoint
  },

  // 检查token是否即将过期（5分钟内）— no longer possible client-side with httpOnly cookies
  isTokenExpiringSoon(_token: string): boolean {
    return false; // rely on backend cookie expiry and 401 handling
  },
};

// 登录状态管理
export const authManager = {
  isAuthenticated(): boolean {
    // Cannot inspect httpOnly cookies from JS; rely on backend 401 responses
    return true;
  },

  logout(): void {
    // 触发登出事件
    window.dispatchEvent(new CustomEvent("auth:logout"));
  },

  onLogout(callback: () => void): () => void {
    const handler = () => callback();
    window.addEventListener("auth:logout", handler);
    return () => window.removeEventListener("auth:logout", handler);
  },
};
