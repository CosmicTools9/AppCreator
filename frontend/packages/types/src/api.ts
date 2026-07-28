/**
 * @description Alioth 标准 API 响应类型
 *
 * 与后端 `alioth_common::ApiResponse<T>` 对齐，
 * 所有成功响应统一返回 `{success, data}` 格式。
 *
 * @template T The type of data in the response
 *
 * @example
 * ```typescript
 * const response: ApiResponse<User> = {
 *   success: true,
 *   data: { id: "1", name: "John" },
 * };
 *
 * // Access typed data
 * console.log(response.data.name);
 * ```
 */
export interface ApiResponse<T> {
  /** 响应是否成功 */
  success: boolean;
  /** Response payload of generic type T */
  data: T;
  /** 可选的附加消息 */
  message?: string;
}

/**
 * @description Paginated data wrapper for list responses
 *
 * 与后端 `alioth_common::PaginatedResponse<T>` 序列化输出对齐。
 * 后端同时输出 `list` / `items` / `pageSize` / `page_size` 等兼容字段。
 *
 * @template T The type of items in the list
 *
 * @example
 * ```typescript
 * const usersResponse: ApiResponse<PaginatedData<User>> = await api.get("/users?page=1&page_size=20");
 * const users = usersResponse.data;
 *
 * console.log(`Showing ${users.list.length} of ${users.total} users`);
 * console.log(`Page ${users.page} of ${users.totalPages}`);
 * ```
 */
export interface PaginatedData<T> {
  /** Array of items for the current page (主字段，与前端组件对齐) */
  list: T[];
  /** Alias for `list` (backward compatibility) */
  items?: T[];
  /** Total number of items across all pages */
  total: number;
  /** Current page number (1-indexed) */
  page: number;
  /** Number of items per page */
  pageSize: number;
  /** Alias for `pageSize` (backward compatibility) */
  page_size?: number;
  /** Total number of pages available */
  totalPages: number;
}

/**
 * @description List query parameters for paginated/filtered/sorted requests
 *
 * 与后端 `alioth_common::ListQuery` 对齐，字段名使用 snake_case
 * 以匹配后端 serde 反序列化期望。
 *
 * @example
 * ```typescript
 * const params: ListQueryParams = {
 *   page: 2,
 *   page_size: 50,
 *   sort_field: "created_at",
 *   sort_order: "desc",
 *   filter_field: "status",
 *   filter_op: "eq",
 *   filter_value: "active",
 * };
 *
 * const response = await api.get("/items", { params });
 * ```
 */
export interface ListQueryParams {
  /** Page number to retrieve (default: 1) */
  page?: number;
  /** Number of items per page (default: 20) */
  page_size?: number;
  /** Field to filter on */
  filter_field?: string;
  /** Filter operator: eq, ne, gt, gte, lt, lte, like, in */
  filter_op?: string;
  /** Filter value */
  filter_value?: string;
  /** Field name to sort by */
  sort_field?: string;
  /** Sort direction: asc or desc */
  sort_order?: "asc" | "desc";
  /** 允许各应用扩展自定义查询参数 */
  [key: string]: unknown;
}

/**
 * @deprecated Use `ListQueryParams` instead.
 * 旧版分页参数（camelCase），仅保留用于兼容已有代码。
 */
export interface PaginationParams {
  /** Page number to retrieve (default: 1) */
  page?: number;
  /** Number of items per page (default: 20) */
  pageSize?: number;
  /** Field name to sort by */
  sortBy?: string;
  /** Sort direction: ascending or descending */
  sortOrder?: "asc" | "desc";
}
