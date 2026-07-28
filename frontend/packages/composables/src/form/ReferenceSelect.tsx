//! ReferenceSelect · FK 关联字段搜索下拉
//!
//! 针对 `EntityFormPage` 中 FK 字段直接输入 raw ID 的问题，
//! 提供基于 API 端点的可搜索关联选择器，代替原生数字输入框。
//! 支持分页（pageSize/page + hasMore）和搜索防抖（350ms debounce）。
//!
//! 使用方式：
//! ```tsx
//! <ReferenceSelect
//!   endpoint="/api/clients/subjects"
//!   labelField="notice"
//!   value={form.fk_subject}
//!   onChange={(id) => setForm({ ...form, fk_subject: id })}
//! />
//! ```

import * as React from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import { apiClient } from "@alioth/api";
import { SearchableSelect, type SearchableSelectOption } from "@alioth/components";
import { useT } from "@alioth/i18n";

// ── 防抖 hook ──
function useDebounce<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = React.useState(value);
  React.useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);
  return debounced;
}

export interface ReferenceSelectProps {
  /** API 端点路径（不含 base URL），如 "/clients/subjects" */
  endpoint: string;
  /** 用于显示的字段名，如 "notice"、"code"、"name" */
  labelField: string;
  /** 当前选中的 FK ID */
  value: string | number | null | undefined;
  /** 选中回调 */
  onChange: (value: string) => void;
  /** 查询时显示的占位符 */
  placeholder?: string;
  /** 搜索占位符 */
  searchPlaceholder?: string;
  /** 禁用 */
  disabled?: boolean;
  /** 附加查询参数 */
  queryParams?: Record<string, string | number | boolean | undefined>;
  /** 分页大小（默认 50） */
  pageSize?: number;
  /** 是否开启搜索防抖（默认 true） */
  debounceSearch?: boolean;
  /** 初始选中项的显示标签（供编辑回填时直接显示 label，无需等待 API 加载） */
  initialLabel?: string;
}

/**
 * ReferenceSelect — 从 API 端点获取关联数据并渲染为可搜索下拉。
 * 支持分页加载（InfiniteQuery）和搜索防抖。
 */
export function ReferenceSelect({
  endpoint,
  labelField,
  value,
  onChange,
  placeholder,
  searchPlaceholder,
  disabled,
  queryParams,
  pageSize = 50,
  debounceSearch = true,
  initialLabel,
}: ReferenceSelectProps) {
  const t = useT();
  const [search, setSearch] = React.useState("");
  const debouncedSearch = useDebounce(search, debounceSearch ? 350 : 0);

  const {
    data,
    isLoading,
    isError,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["reference-select", endpoint, queryParams, debouncedSearch],
    queryFn: async ({ pageParam = 1 }) => {
      const params: Record<string, unknown> = {
        ...queryParams,
        page: pageParam,
        page_size: pageSize,
      };
      if (debouncedSearch) {
        params.search = debouncedSearch;
      }
      const res = await apiClient.get(endpoint, { params });
      const responseData = (res as Record<string, unknown>)?.data as Record<string, unknown> | undefined;
      const body = responseData ?? (res as Record<string, unknown>);
      const rawItems = (body?.items as unknown) ?? (body?.list as unknown) ?? (body?.data as unknown) ?? body;
      const itemsArray: Record<string, unknown>[] = Array.isArray(rawItems) ? (rawItems as Record<string, unknown>[]) : [];
      const currentPage = typeof body?.page === "number" ? body.page : pageParam;
      const total = typeof body?.total === "number" ? body.total : itemsArray.length;
      const respPageSize = typeof body?.page_size === "number" ? body.page_size : pageSize;
      const totalPages = respPageSize > 0 ? Math.ceil(total / respPageSize) : 1;
      return {
        items: itemsArray,
        nextPage: currentPage + 1,
        hasMore: currentPage < totalPages,
      };
    },
    initialPageParam: 1,
    getNextPageParam: (lastPage) =>
      lastPage.hasMore ? lastPage.nextPage : undefined,
    staleTime: 30_000,
    retry: 1,
  });

  const options: SearchableSelectOption[] = React.useMemo(() => {
    if (!data?.pages) return [];
    const opts = data.pages.flatMap((page) =>
      (page.items ?? []).map((item) => {
        const rec = item as Record<string, unknown>;
        return {
          value: String(rec.id),
          label: String(
            rec[labelField] ??
              rec.notice ??
              rec.code ??
              rec.name ??
              rec.id
          ),
        };
      })
    );
    // 如果已有 options 中不包含当前选中项，但提供了 initialLabel，则注入
    if (
      value != null &&
      initialLabel &&
      !opts.find((o) => o.value === String(value))
    ) {
      opts.unshift({ value: String(value), label: initialLabel });
    }
    return opts;
  }, [data, labelField, value, initialLabel]);

  const handleScrollEnd = React.useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) {
      fetchNextPage();
    }
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  if (isError) {
    return (
      <div className="flex items-center gap-2 text-xs text-destructive">
        <span className="rounded px-2 py-1 bg-destructive/10 border border-destructive/20">
          {t("common.loadFailed", {}, { fallback: "加载失败" })}
        </span>
      </div>
    );
  }

  return (
    <SearchableSelect
      options={options}
      value={value != null ? String(value) : ""}
      onChange={onChange}
      placeholder={
        isLoading
          ? t("common.loading", {}, { fallback: "加载中..." })
          : (placeholder ?? t("common.pleaseSelect", {}, { fallback: "请选择" }))
      }
      searchPlaceholder={
        searchPlaceholder ??
        t("common.searchAndSelect", {}, { fallback: "搜索并选择..." })
      }
      disabled={disabled || isLoading}
      onSearchChange={setSearch}
      onScrollEnd={handleScrollEnd}
      loadingMore={isFetchingNextPage}
    />
  );
}
