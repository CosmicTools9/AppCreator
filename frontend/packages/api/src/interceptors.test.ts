import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  ErrorType,
  categorizeError,
  calculateRetryDelay,
  RetryInterceptor,
  defaultRetryConfig,
  type CategorizedApiError,
  type ApiError,
} from "./interceptors.js";

describe("categorizeError", () => {
  it("should categorize timeout error by code", () => {
    const error: ApiError = { code: "TIMEOUT", message: "Request timeout" };
    const result = categorizeError(error);

    expect(result.type).toBe(ErrorType.TIMEOUT_ERROR);
    expect(result.retryable).toBe(true);
  });

  it("should categorize timeout error by message", () => {
    const error: ApiError = {
      code: "UNKNOWN",
      message: "Request timeout occurred",
    };
    const result = categorizeError(error);

    expect(result.type).toBe(ErrorType.TIMEOUT_ERROR);
    expect(result.retryable).toBe(true);
  });

  it("should categorize network error by code", () => {
    const error: ApiError = { code: "NETWORK_ERROR", message: "Network error" };
    const result = categorizeError(error);

    expect(result.type).toBe(ErrorType.NETWORK_ERROR);
    expect(result.retryable).toBe(true);
  });

  it("should categorize network error by message", () => {
    const error: ApiError = { code: "UNKNOWN", message: "Failed to fetch" };
    const originalError = new TypeError("Failed to fetch");
    const result = categorizeError(error, undefined, originalError);

    expect(result.type).toBe(ErrorType.NETWORK_ERROR);
    expect(result.retryable).toBe(true);
  });

  it("should categorize 500 errors as HTTP_ERROR with retryable based on status code", () => {
    const error: ApiError = {
      code: "HTTP_500",
      message: "Internal server error",
    };
    const result = categorizeError(error, 500);

    expect(result.type).toBe(ErrorType.HTTP_ERROR);
    expect(result.retryable).toBe(true);
  });

  it("should categorize 401 errors as AUTH_ERROR and not retryable", () => {
    const error: ApiError = { code: "UNAUTHORIZED", message: "Unauthorized" };
    const result = categorizeError(error, 401);

    expect(result.type).toBe(ErrorType.AUTH_ERROR);
    expect(result.retryable).toBe(false);
  });

  it("should categorize 400 errors as VALIDATION_ERROR and not retryable", () => {
    const error: ApiError = { code: "BAD_REQUEST", message: "Bad request" };
    const result = categorizeError(error, 400);

    expect(result.type).toBe(ErrorType.VALIDATION_ERROR);
    expect(result.retryable).toBe(false);
  });

  it("should categorize 404 errors as HTTP_ERROR and not retryable", () => {
    const error: ApiError = { code: "NOT_FOUND", message: "Not found" };
    const result = categorizeError(error, 404);

    expect(result.type).toBe(ErrorType.HTTP_ERROR);
    expect(result.retryable).toBe(false);
  });
});

describe("calculateRetryDelay", () => {
  it("should calculate exponential backoff delay", () => {
    const config = { baseDelay: 1000, maxDelay: 30000, backoffMultiplier: 2 };

    expect(calculateRetryDelay(1, config)).toBe(1000); // 1000 * 2^0
    expect(calculateRetryDelay(2, config)).toBe(2000); // 1000 * 2^1
    expect(calculateRetryDelay(3, config)).toBe(4000); // 1000 * 2^2
    expect(calculateRetryDelay(4, config)).toBe(8000); // 1000 * 2^3
  });

  it("should cap delay at maxDelay", () => {
    const config = { baseDelay: 1000, maxDelay: 5000, backoffMultiplier: 2 };

    expect(calculateRetryDelay(10, config)).toBe(5000);
  });

  it("should use default config when not provided", () => {
    const delay1 = calculateRetryDelay(1);
    const delay2 = calculateRetryDelay(2);

    expect(delay1).toBe(1000);
    expect(delay2).toBe(2000);
  });
});

describe("RetryInterceptor", () => {
  let interceptor: RetryInterceptor;

  beforeEach(() => {
    interceptor = new RetryInterceptor({
      maxRetries: 3,
      baseDelay: 100,
      maxDelay: 1000,
      backoffMultiplier: 2,
      retryableStatusCodes: [500, 502, 503, 504],
    });
  });

  it("should retry on TIMEOUT_ERROR", () => {
    const error: CategorizedApiError = {
      code: "TIMEOUT",
      message: "Timeout",
      type: ErrorType.TIMEOUT_ERROR,
      retryable: true,
    };
    const config = { url: "/api/test", method: "GET" };

    expect(interceptor.shouldRetry(error, config)).toBe(true);
  });

  it("should retry on NETWORK_ERROR", () => {
    const error: CategorizedApiError = {
      code: "NETWORK_ERROR",
      message: "Network error",
      type: ErrorType.NETWORK_ERROR,
      retryable: true,
    };
    const config = { url: "/api/test", method: "GET" };

    expect(interceptor.shouldRetry(error, config)).toBe(true);
  });

  it("should retry on HTTP_ERROR with 5xx status code", () => {
    const error: CategorizedApiError = {
      code: "HTTP_500",
      message: "Internal server error",
      type: ErrorType.HTTP_ERROR,
      statusCode: 500,
      retryable: true,
    };
    const config = { url: "/api/test", method: "GET" };

    expect(interceptor.shouldRetry(error, config)).toBe(true);
  });

  it("should NOT retry on AUTH_ERROR", () => {
    const error: CategorizedApiError = {
      code: "UNAUTHORIZED",
      message: "Unauthorized",
      type: ErrorType.AUTH_ERROR,
      statusCode: 401,
      retryable: false,
    };
    const config = { url: "/api/test", method: "GET" };

    expect(interceptor.shouldRetry(error, config)).toBe(false);
  });

  it("should NOT retry on VALIDATION_ERROR", () => {
    const error: CategorizedApiError = {
      code: "BAD_REQUEST",
      message: "Bad request",
      type: ErrorType.VALIDATION_ERROR,
      statusCode: 400,
      retryable: false,
    };
    const config = { url: "/api/test", method: "GET" };

    expect(interceptor.shouldRetry(error, config)).toBe(false);
  });

  it("should NOT retry after max retries reached", () => {
    const error: CategorizedApiError = {
      code: "HTTP_500",
      message: "Internal server error",
      type: ErrorType.HTTP_ERROR,
      statusCode: 500,
      retryable: true,
    };
    const config = { url: "/api/test", method: "GET" };

    // Simulate 3 retries
    interceptor.incrementRetryCount(config);
    interceptor.incrementRetryCount(config);
    interceptor.incrementRetryCount(config);

    expect(interceptor.shouldRetry(error, config)).toBe(false);
  });

  it("should track retry count per request", () => {
    const error: CategorizedApiError = {
      code: "HTTP_500",
      message: "Internal server error",
      type: ErrorType.HTTP_ERROR,
      statusCode: 500,
      retryable: true,
    };
    const config1 = { url: "/api/test1", method: "GET" };
    const config2 = { url: "/api/test2", method: "GET" };

    interceptor.incrementRetryCount(config1);

    expect(interceptor.getRetryCount(config1)).toBe(1);
    expect(interceptor.getRetryCount(config2)).toBe(0);
  });

  it("should reset retry count after success", () => {
    const config = { url: "/api/test", method: "GET" };

    interceptor.incrementRetryCount(config);
    expect(interceptor.getRetryCount(config)).toBe(1);

    interceptor.resetRetryCount(config);
    expect(interceptor.getRetryCount(config)).toBe(0);
  });

  it("should wait for retry delay", async () => {
    const startTime = Date.now();
    await interceptor.waitForRetry(1);
    const elapsed = Date.now() - startTime;

    // Should wait approximately 200ms (100 * 2^1)
    expect(elapsed).toBeGreaterThanOrEqual(150);
    expect(elapsed).toBeLessThan(300);
  });

  it("should clear all retry counts", () => {
    const config1 = { url: "/api/test1", method: "GET" };
    const config2 = { url: "/api/test2", method: "GET" };

    interceptor.incrementRetryCount(config1);
    interceptor.incrementRetryCount(config2);

    interceptor.clearAll();

    expect(interceptor.getRetryCount(config1)).toBe(0);
    expect(interceptor.getRetryCount(config2)).toBe(0);
  });
});
