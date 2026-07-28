/**
 * React Query Hooks Factory
 *
 * Factory functions for creating type-safe React Query hooks
 * with standardized error handling and caching.
 */

import * as React from "react";
import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
  type UseMutationOptions,
  type UseQueryResult,
  type QueryKey,
  type QueryFunction,
} from "@tanstack/react-query";
import type { ApiClient } from "./client.js";
import type { PaginatedData, PaginationParams, ApiResponse } from "./types.js";

// ============================================
// Types
// ============================================

export interface QueryHookConfig<TData, TError = Error> {
  /** Query key or function to generate key */
  queryKey: QueryKey | ((...args: unknown[]) => QueryKey);
  /** Query function */
  queryFn: QueryFunction<TData>;
  /** Default options for useQuery */
  defaultOptions?: Omit<UseQueryOptions<TData, TError>, "queryKey" | "queryFn">;
}

export interface MutationHookConfig<TData, TVariables, TError = Error> {
  /** Mutation key */
  mutationKey?: QueryKey;
  /** Mutation function */
  mutationFn: (variables: TVariables) => Promise<TData>;
  /** Query keys to invalidate on success */
  invalidateQueries?: QueryKey[];
  /** Default options for useMutation */
  defaultOptions?: Omit<
    UseMutationOptions<TData, TError, TVariables>,
    "mutationKey" | "mutationFn"
  >;
}

export interface ApiQueryOptions<TData, TError = Error> extends Omit<
  UseQueryOptions<TData, TError>,
  "queryKey" | "queryFn"
> {
  /** API endpoint path */
  endpoint: string;
  /** Query parameters */
  params?: Record<string, unknown>;
  /** Whether to skip the request */
  skip?: boolean;
}

export interface ApiMutationOptions<
  TData,
  TVariables,
  TError = Error,
> extends Omit<
  UseMutationOptions<TData, TError, TVariables>,
  "mutationKey" | "mutationFn"
> {
  /** API endpoint path (can include {placeholders}) */
  endpoint: string | ((variables: TVariables) => string);
  /** HTTP method */
  method?: "POST" | "PUT" | "PATCH" | "DELETE";
  /** Query keys to invalidate on success */
  invalidateQueries?: QueryKey[];
}

// ============================================
// Query Hook Factory
// ============================================

/**
 * Creates a type-safe query hook
 *
 * @example
 * ```typescript
 * const useUser = createQueryHook<User, { id: string }>({
 *   queryKey: (id) => ["users", id],
 *   queryFn: (id) => apiClient.get(`/users/${id}`),
 * });
 *
 * const { data, isLoading } = useUser({ id: "123" });
 * ```
 */
export function createQueryHook<TData, TParams = void, TError = Error>(
  config: QueryHookConfig<TData, TError> & {
    queryKey: (...args: TParams extends void ? [] : [TParams]) => QueryKey;
    queryFn: TParams extends void
      ? QueryFunction<TData>
      : (params: TParams) => Promise<TData>;
  },
) {
  return function useTypedQuery(
    ...args: TParams extends void
      ? [options?: Omit<UseQueryOptions<TData, TError>, "queryKey" | "queryFn">]
      : [
          params: TParams,
          options?: Omit<
            UseQueryOptions<TData, TError>,
            "queryKey" | "queryFn"
          >,
        ]
  ) {
    const [paramsOrOptions, maybeOptions] = args;

    const params =
      typeof paramsOrOptions === "object" &&
      paramsOrOptions !== null &&
      !("queryKey" in paramsOrOptions)
        ? (paramsOrOptions as TParams)
        : undefined;

    const options =
      maybeOptions ||
      ((typeof paramsOrOptions === "object" &&
      "queryKey" in (paramsOrOptions || {})
        ? paramsOrOptions
        : {}) as Omit<UseQueryOptions<TData, TError>, "queryKey" | "queryFn">);

    const queryKey = params
      ? (config.queryKey as (p: TParams) => QueryKey)(params)
      : (config.queryKey as () => QueryKey)();

    const queryFn: QueryFunction<TData> = params
      ? () => (config.queryFn as (params: TParams) => Promise<TData>)(params)
      : (config.queryFn as QueryFunction<TData>);

    return useQuery<TData, TError>({
      queryKey,
      queryFn,
      ...config.defaultOptions,
      ...options,
    });
  };
}

/**
 * Creates a query hook from an API client method
 *
 * @example
 * ```typescript
 * const useUsers = createApiQueryHook<User[]>({
 *   endpoint: "/users",
 *   queryKeyBase: "users",
 * });
 * ```
 */
export function createApiQueryHook<
  TData,
  TParams = Record<string, unknown>,
>(config: {
  client: ApiClient;
  endpoint: string | ((params: TParams) => string);
  queryKeyBase: string;
  defaultOptions?: Omit<UseQueryOptions<TData, Error>, "queryKey" | "queryFn">;
}) {
  const { client, endpoint, queryKeyBase, defaultOptions } = config;

  return function useApiQuery(
    params?: TParams,
    options?: Omit<UseQueryOptions<TData, Error>, "queryKey" | "queryFn">,
  ) {
    const queryKey = params ? [queryKeyBase, params] : [queryKeyBase];

    const url =
      typeof endpoint === "function" ? endpoint(params as TParams) : endpoint;

    return useQuery<TData, Error>({
      queryKey,
      queryFn: () => client.get<TData>(url),
      ...defaultOptions,
      ...options,
    });
  };
}

// ============================================
// Mutation Hook Factory
// ============================================

/**
 * Creates a type-safe mutation hook
 *
 * @example
 * ```typescript
 * const useCreateUser = createMutationHook<User, CreateUserInput>({
 *   mutationFn: (data) => apiClient.post("/users", data),
 *   invalidateQueries: [["users"]],
 * });
 *
 * const { mutate, isPending } = useCreateUser();
 * mutate({ name: "John" });
 * ```
 */
export function createMutationHook<TData, TVariables = void, TError = Error>(
  config: MutationHookConfig<TData, TVariables, TError>,
) {
  return function useTypedMutation(
    options?: Omit<
      UseMutationOptions<TData, TError, TVariables>,
      "mutationKey" | "mutationFn"
    >,
  ) {
    const queryClient = useQueryClient();

    return useMutation<TData, TError, TVariables>({
      mutationKey: config.mutationKey,
      mutationFn: config.mutationFn,
      ...config.defaultOptions,
      ...options,
      onSuccess: (data, variables) => {
        // Invalidate specified queries
        if (config.invalidateQueries) {
          config.invalidateQueries.forEach((queryKey) => {
            void queryClient.invalidateQueries({ queryKey } as never);
          });
        }
        // Call user's onSuccess
        const onSuccess = options?.onSuccess as
          | ((data: TData, variables: TVariables) => void)
          | undefined;
        if (onSuccess) {
          onSuccess(data, variables);
        }
      },
    });
  };
}

/**
 * Creates a mutation hook from an API client
 *
 * @example
 * ```typescript
 * const useCreateUser = createApiMutationHook<User, CreateUserInput>({
 *   client,
 *   endpoint: "/users",
 *   method: "POST",
 *   invalidateQueries: [["users"]],
 * });
 * ```
 */
export function createApiMutationHook<
  TData,
  TVariables = Record<string, unknown>,
>(config: {
  client: ApiClient;
  endpoint: string | ((variables: TVariables) => string);
  method?: "POST" | "PUT" | "PATCH" | "DELETE";
  invalidateQueries?: QueryKey[];
  defaultOptions?: Omit<
    UseMutationOptions<TData, Error, TVariables>,
    "mutationKey" | "mutationFn"
  >;
}) {
  const {
    client,
    endpoint,
    method = "POST",
    invalidateQueries,
    defaultOptions,
  } = config;

  return function useApiMutation(
    options?: Omit<
      UseMutationOptions<TData, Error, TVariables>,
      "mutationKey" | "mutationFn"
    >,
  ) {
    const queryClient = useQueryClient();

    return useMutation<TData, Error, TVariables>({
      mutationFn: async (variables) => {
        const url =
          typeof endpoint === "function" ? endpoint(variables) : endpoint;

        switch (method) {
          case "POST":
            return client.post<TData>(url, variables);
          case "PUT":
            return client.put<TData>(url, variables);
          case "PATCH":
            return client.patch<TData>(url, variables);
          case "DELETE":
            return client.delete<TData>(url);
          default:
            throw new Error(`Unsupported method: ${method}`);
        }
      },
      ...defaultOptions,
      ...options,
      onSuccess: (data, variables) => {
        // Invalidate specified queries
        invalidateQueries?.forEach((queryKey) => {
          void queryClient.invalidateQueries({ queryKey } as never);
        });
        // Call user's onSuccess
        const onSuccess = options?.onSuccess as
          | ((data: TData, variables: TVariables) => void)
          | undefined;
        if (onSuccess) {
          onSuccess(data, variables);
        }
      },
    });
  };
}

// ============================================
// CRUD Hooks Factory
// ============================================

export interface CrudHooksConfig<T, TCreate, TUpdate, TId = string> {
  client: ApiClient;
  baseEndpoint: string;
  queryKeyBase: string;
}

export interface CrudHooks<T, TCreate, TUpdate, TId = string> {
  useList: (
    options?: Omit<UseQueryOptions<T[], Error>, "queryKey" | "queryFn">,
  ) => ReturnType<typeof useQuery<T[], Error>>;
  useGet: (
    id: TId,
    options?: Omit<UseQueryOptions<T, Error>, "queryKey" | "queryFn">,
  ) => ReturnType<typeof useQuery<T, Error>>;
  useCreate: (
    options?: Omit<
      UseMutationOptions<T, Error, TCreate>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<T, Error, TCreate>>;
  useUpdate: (
    options?: Omit<
      UseMutationOptions<T, Error, { id: TId; data: TUpdate }>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<T, Error, { id: TId; data: TUpdate }>>;
  useDelete: (
    options?: Omit<
      UseMutationOptions<void, Error, TId>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<void, Error, TId>>;
  useBatchDelete: (
    options?: Omit<
      UseMutationOptions<void, Error, TId[]>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<void, Error, TId[]>>;
}

/**
 * Creates a complete set of CRUD hooks for an entity
 *
 * @example
 * ```typescript
 * const userHooks = createCrudHooks<User, CreateUserInput, UpdateUserInput>({
 *   client,
 *   baseEndpoint: "/users",
 *   queryKeyBase: "users",
 * });
 *
 * // Usage
 * const { data: users } = userHooks.useList();
 * const { data: user } = userHooks.useGet("123");
 * const create = userHooks.useCreate();
 * ```
 */
export function createCrudHooks<T, TCreate, TUpdate, TId = string>(
  config: CrudHooksConfig<T, TCreate, TUpdate, TId>,
): CrudHooks<T, TCreate, TUpdate, TId> {
  const { client, baseEndpoint, queryKeyBase } = config;

  const listQueryKey: QueryKey = [queryKeyBase];
  const detailQueryKey = (id: TId): QueryKey => [queryKeyBase, id];

  return {
    useList: (options) =>
      useQuery<T[], Error>({
        queryKey: listQueryKey,
        queryFn: () => client.get<T[]>(baseEndpoint),
        ...options,
      }),

    useGet: (id, options) =>
      useQuery<T, Error>({
        queryKey: detailQueryKey(id),
        queryFn: () => client.get<T>(`${baseEndpoint}/${id}`),
        enabled: !!id,
        ...options,
      }),

    useCreate: (options) => {
      const queryClient = useQueryClient();
      return useMutation<T, Error, TCreate>({
        mutationFn: (data) => client.post<T>(baseEndpoint, data),
        ...options,
        onSuccess: (data, variables) => {
          void queryClient.invalidateQueries({
            queryKey: listQueryKey,
          } as never);
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: T, variables: TCreate) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, variables);
          }
        },
      });
    },

    useUpdate: (options) => {
      const queryClient = useQueryClient();
      return useMutation<T, Error, { id: TId; data: TUpdate }>({
        mutationFn: ({ id, data }) =>
          client.put<T>(`${baseEndpoint}/${id}`, data),
        ...options,
        onSuccess: (data, variables) => {
          void queryClient.invalidateQueries({
            queryKey: listQueryKey,
          } as never);
          void queryClient.invalidateQueries({
            queryKey: detailQueryKey(variables.id),
          } as never);
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: T, variables: { id: TId; data: TUpdate }) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, variables);
          }
        },
      });
    },

    useDelete: (options) => {
      const queryClient = useQueryClient();
      return useMutation<void, Error, TId>({
        mutationFn: (id) => client.delete<void>(`${baseEndpoint}/${id}`),
        ...options,
        onSuccess: (data, id) => {
          void queryClient.invalidateQueries({
            queryKey: listQueryKey,
          } as never);
          void queryClient.removeQueries({
            queryKey: detailQueryKey(id),
          } as never);
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: void, variables: TId) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, id);
          }
        },
      });
    },

    useBatchDelete: (options) => {
      const queryClient = useQueryClient();
      return useMutation<void, Error, TId[]>({
        mutationFn: (ids) => client.delete<void>(`${baseEndpoint}/batch`, ids),
        ...options,
        onSuccess: (data, ids) => {
          void queryClient.invalidateQueries({
            queryKey: [queryKeyBase],
          } as never);
          for (const id of ids) {
            void queryClient.removeQueries({
              queryKey: detailQueryKey(id),
            } as never);
          }
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: void, variables: TId[]) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, ids);
          }
        },
      });
    },
  };
}

// ============================================
// Wrapped Paginated CRUD Hooks Factory
// ============================================

export interface PaginatedCrudHooksConfig<T, TCreate, TUpdate, TId = string> {
  client: ApiClient;
  baseEndpoint: string;
  queryKeyBase: string | string[];
}

export interface PaginatedCrudHooks<T, TCreate, TUpdate, TId = string> {
  useList: (
    params?: PaginationParams,
    options?: Omit<
      UseQueryOptions<PaginatedData<T>, Error>,
      "queryKey" | "queryFn"
    >,
  ) => UseQueryResult<PaginatedData<T>, Error>;
  useGet: (
    id: TId,
    options?: Omit<UseQueryOptions<T, Error>, "queryKey" | "queryFn">,
  ) => ReturnType<typeof useQuery<T, Error>>;
  useCreate: (
    options?: Omit<
      UseMutationOptions<T, Error, TCreate>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<T, Error, TCreate>>;
  useUpdate: (
    options?: Omit<
      UseMutationOptions<T, Error, { id: TId; data: TUpdate }>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<T, Error, { id: TId; data: TUpdate }>>;
  useDelete: (
    options?: Omit<
      UseMutationOptions<void, Error, TId>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<void, Error, TId>>;
  useBatchDelete: (
    options?: Omit<
      UseMutationOptions<void, Error, TId[]>,
      "mutationKey" | "mutationFn"
    >,
  ) => ReturnType<typeof useMutation<void, Error, TId[]>>;
}

/**
 * 创建支持后端 ApiResponse 包装层的 CRUD Hooks 工厂
 *
 * 假设后端返回 `{success: true, data: PaginatedData<T>}` / `{success: true, data: T}` 格式
 * 自动解包 `.data` 字段
 *
 * @example
 * ```ts
 * const inventoryHooks = createPaginatedCrudHooks<Inventory, CreateInventory, UpdateInventory, number>({
 *   client: apiClient,
 *   baseEndpoint: "/inventories",
 *   queryKeyBase: "inventories",
 * });
 * ```
 */
export function createPaginatedCrudHooks<T, TCreate, TUpdate, TId = string>(
  config: PaginatedCrudHooksConfig<T, TCreate, TUpdate, TId>,
): PaginatedCrudHooks<T, TCreate, TUpdate, TId> {
  const { client, baseEndpoint, queryKeyBase } = config;

  const listQueryKey: QueryKey = Array.isArray(queryKeyBase) ? queryKeyBase : [queryKeyBase];
  const detailQueryKey = (id: TId): QueryKey => Array.isArray(queryKeyBase) ? [...queryKeyBase, id] : [queryKeyBase, id];

  const unwrap = <R,>(response: ApiResponse<R>): R => response.data;

  return {
    useList: (params, options) => {
      const queryKey: QueryKey = params
        ? [...listQueryKey, JSON.stringify(params)]
        : listQueryKey;

      return useQuery<PaginatedData<T>, Error>({
        queryKey,
        queryFn: async () => {
          const res = await client.get<ApiResponse<PaginatedData<T>>>(
            baseEndpoint,
            { params: params ?? {} },
          );
          return unwrap(res);
        },
        ...options,
      });
    },

    useGet: (id, options) =>
      useQuery<T, Error>({
        queryKey: detailQueryKey(id),
        queryFn: async () => {
          const res = await client.get<ApiResponse<T>>(`${baseEndpoint}/${id}`);
          return unwrap(res);
        },
        enabled: !!id,
        ...options,
      }),

    useCreate: (options) => {
      const queryClient = useQueryClient();
      return useMutation<T, Error, TCreate>({
        mutationFn: async (data) => {
          const res = await client.post<ApiResponse<T>>(baseEndpoint, data);
          return unwrap(res);
        },
        ...options,
        onSuccess: (data, variables) => {
          void queryClient.invalidateQueries({
            queryKey: listQueryKey,
          } as never);
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: T, variables: TCreate) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, variables);
          }
        },
      });
    },

    useUpdate: (options) => {
      const queryClient = useQueryClient();
      return useMutation<T, Error, { id: TId; data: TUpdate }>({
        mutationFn: async ({ id, data }) => {
          const res = await client.put<ApiResponse<T>>(`${baseEndpoint}/${id}`, data);
          return unwrap(res);
        },
        ...options,
        onSuccess: (data, variables) => {
          void queryClient.invalidateQueries({
            queryKey: listQueryKey,
          } as never);
          void queryClient.invalidateQueries({
            queryKey: detailQueryKey(variables.id),
          } as never);
          client.invalidateCachePattern(`${baseEndpoint}*`);
          const onSuccess = options?.onSuccess as
            | ((data: T, variables: { id: TId; data: TUpdate }) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, variables);
          }
        },
      });
    },

    useDelete: (options) => {
      const queryClient = useQueryClient();
      return useMutation<void, Error, TId>({
        mutationFn: async (id) => {
          await client.delete<ApiResponse<void>>(`${baseEndpoint}/${id}`);
        },
        ...options,
        onSuccess: async (data, id) => {
          client.invalidateCachePattern(`${baseEndpoint}*`);
          await queryClient.invalidateQueries({
            queryKey: listQueryKey,
            refetchType: "all",
          } as never);
          queryClient.removeQueries({
            queryKey: detailQueryKey(id),
          } as never);
          const onSuccess = options?.onSuccess as
            | ((data: void, variables: TId) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, id);
          }
        },
      });
    },

    useBatchDelete: (options) => {
      const queryClient = useQueryClient();
      return useMutation<void, Error, TId[]>({
        mutationFn: async (ids) => {
          await client.delete<ApiResponse<void>>(`${baseEndpoint}/batch`, ids);
        },
        ...options,
        onSuccess: async (data, ids) => {
          client.invalidateCachePattern(`${baseEndpoint}*`);
          await queryClient.invalidateQueries({
            queryKey: Array.isArray(queryKeyBase) ? queryKeyBase : [queryKeyBase],
            refetchType: "all",
          } as never);
          for (const id of ids) {
            queryClient.removeQueries({
              queryKey: detailQueryKey(id),
            } as never);
          }
          const onSuccess = options?.onSuccess as
            | ((data: void, variables: TId[]) => void)
            | undefined;
          if (onSuccess) {
            onSuccess(data, ids);
          }
        },
      });
    },
  };
}


