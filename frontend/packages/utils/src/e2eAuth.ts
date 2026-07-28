import { createHmac } from "crypto";

/**
 * Create a test JWT token for E2E API testing.
 * Uses HS256 algorithm with the shared dev JWT secret.
 */
export function createTestJWT(
  payload: Record<string, unknown> = {},
  secret: string = "dev-secret-key-change-in-production-min-32-chars"
): string {
  const header = Buffer.from(
    JSON.stringify({ alg: "HS256", typ: "JWT" })
  ).toString("base64url");

  const defaultPayload = {
    sub: "1",
    username: "e2e-admin",
    email: "e2e-admin@alioth.test",
    is_superuser: true,
    role: "admin",
    iat: Math.floor(Date.now() / 1000),
    exp: Math.floor(Date.now() / 1000) + 86400,
    ...payload,
  };

  const body = Buffer.from(JSON.stringify(defaultPayload)).toString(
    "base64url"
  );

  const signature = createHmac("sha256", secret)
    .update(`${header}.${body}`)
    .digest("base64url");

  return `${header}.${body}.${signature}`;
}

/**
 * Default Authorization header value for E2E tests.
 */
export function getTestAuthHeader(
  payload?: Record<string, unknown>
): Record<string, string> {
  return {
    Authorization: `Bearer ${createTestJWT(payload)}`,
  };
}
