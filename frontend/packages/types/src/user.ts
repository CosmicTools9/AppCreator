/**
 * @description User entity representing a registered user in the system
 *
 * Contains user profile information, roles, and permissions for authorization.
 * Used across all frontend applications for user state management.
 *
 * @example
 * ```typescript
 * const user: User = {
 *   id: "550e8400-e29b-41d4-a716-446655440000",
 *   username: "john.doe",
 *   email: "john@example.com",
 *   roles: ["admin", "user"],
 *   permissions: ["read", "write", "delete"],
 *   createdAt: "2024-01-01T00:00:00Z",
 *   updatedAt: "2024-01-15T12:00:00Z"
 * };
 * ```
 */
export interface User {
  /** Unique user identifier (UUID) */
  id: string;
  /** Username for display and login */
  username: string;
  /** User's email address */
  email: string;
  /** Optional avatar image URL */
  avatar?: string;
  /** User roles (e.g., "admin", "user", "manager") */
  roles: string[];
  /** User permissions (e.g., "read", "write", "delete") */
  permissions: string[];
  /** Account creation timestamp (ISO 8601) */
  createdAt: string;
  /** Last update timestamp (ISO 8601) */
  updatedAt: string;
  /** NGAC 用户属性列表（由后端 /auth/me 返回），新设计优先使用此字段 */
  ngacUserAttributes?: string[];
  /** 有权访问的模块 ID 列表（由后端 /auth/me 返回） */
  accessibleModules?: string[];
}

/**
 * @description User login credentials for authentication
 *
 * Used in login forms and API requests to authenticate users.
 *
 * @example
 * ```typescript
 * const credentials: LoginCredentials = {
 *   username: "john.doe",
 *   password: "securePassword123"
 * };
 *
 * const response = await api.post("/auth/login", credentials);
 * ```
 */
export interface LoginCredentials {
  /** Username or email for login */
  username: string;
  /** User's password (will be hashed before transmission) */
  password: string;
}

/**
 * @description Authenticated user with session tokens
 *
 * Extends User with authentication tokens and expiration information.
 * Returned from successful login and used for maintaining user sessions.
 *
 * @example
 * ```typescript
 * const authUser: AuthUser = await login(credentials);
 *
 * // Store tokens securely
 * localStorage.setItem("accessToken", authUser.accessToken);
 * localStorage.setItem("refreshToken", authUser.refreshToken);
 *
 * // Check if token is still valid
 * const isExpired = Date.now() > authUser.expiresAt;
 * ```
 */
export interface AuthUser extends User {
  /** JWT access token for API authentication */
  accessToken: string;
  /** Refresh token for obtaining new access tokens */
  refreshToken: string;
  /** Token expiration timestamp (Unix milliseconds) */
  expiresAt: number;
}
