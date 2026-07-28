//! 系统配置页面工厂
//!
//! 消除各模块 SystemConfigPage 的复制粘贴。

import { useState, useMemo, useCallback } from "react";
import { SystemConfigPanel } from "@alioth/components";
import { useT } from "@alioth/i18n";
import { apiClient } from "@alioth/api";
import type { ApiResponse } from "@alioth/api";
import { useQuery, useMutation } from "@tanstack/react-query";
import type {
  ConfigCategory,
  ConfigCategoryCode,
  SystemConfig,
  CreateSystemConfigRequest,
  UpdateSystemConfigRequest,
} from "@alioth/components";


export interface SystemConfigPageOptions {
  /** 模块标识（如 "clients"、"vendors"），用于 domain_ 和 i18n key */
  moduleName: string;
  /** 页面标题 i18n key，默认 `${moduleName}.systemConfig.title` */
  titleKey?: string;
  /** 页面副标题 i18n key，默认 `${moduleName}.systemConfig.subtitle` */
  subtitleKey?: string;
}

/**
 * 创建模块系统配置页面
 *
 * @example
 * ```tsx
 * // pages/SystemConfigPage.tsx
 * import { createSystemConfigPage } from "@alioth/components/system-config";
 * export default createSystemConfigPage({ moduleName: "clients" });
 * ```
 */
export function createSystemConfigPage(options: SystemConfigPageOptions) {
  const {
    moduleName,
    titleKey = `${moduleName}.systemConfig.title`,
    subtitleKey = `${moduleName}.systemConfig.subtitle`,
  } = options;

  return function SystemConfigPage() {
    const t = useT();

    // 从后端 API 获取配置分类 Schema
    const { data: categoriesResp, isLoading: categoriesLoading } =
      useQuery<ApiResponse<{ categories: ConfigCategory[] }>>({
        queryKey: ["system-config", "categories"],
        queryFn: async () => {
          const res = await apiClient.get<ApiResponse<{ categories: ConfigCategory[] }>>("/system-config/categories");
          return res;
        },
      });

    const categories = useMemo<ConfigCategory[]>(
      () => categoriesResp?.data?.categories ?? [],
      [categoriesResp],
    );

    // 从后端 API 获取配置列表
    const { data: configsResp, isLoading: listLoading, refetch } =
      useQuery<ApiResponse<SystemConfig[]>>({
        queryKey: ["system-config"],
        queryFn: async () => {
          const res = await apiClient.get<ApiResponse<SystemConfig[]>>("/system-config");
          return res;
        },
      });

    const configs = useMemo<SystemConfig[]>(
      () => configsResp?.data ?? [],
      [configsResp],
    );

    const [activeCategory, setActiveCategory] = useState<ConfigCategoryCode>("llm");

    // 创建配置
    const { mutateAsync: createConfig, isPending: createLoading } =
      useMutation<SystemConfig, Error, CreateSystemConfigRequest>({
        mutationFn: async (req) => {
          const resp = await apiClient.post<ApiResponse<SystemConfig>>("/system-config", req);
          return resp.data;
        },
        onSuccess: () => refetch(),
      });
    
    // 更新配置
    const { mutateAsync: updateConfig, isPending: updateLoading } =
      useMutation<SystemConfig, Error, { id: string; req: UpdateSystemConfigRequest }>({
        mutationFn: async ({ id, req }) => {
          const resp = await apiClient.put<ApiResponse<SystemConfig>>(`/system-config/${id}`, req);
          return resp.data;
        },
        onSuccess: () => refetch(),
      });
    
    // 删除配置
    const { mutateAsync: deleteConfig, isPending: deleteLoading } =
      useMutation<void, Error, string>({
        mutationFn: async (id) => {
          await apiClient.delete<ApiResponse<void>>(`/system-config/${id}`);
        },
        onSuccess: () => refetch(),
      });
    
    const loading = categoriesLoading || listLoading || createLoading || updateLoading || deleteLoading;
    
    const handleCreate = useCallback(
      async (req: CreateSystemConfigRequest) => {
        await createConfig(req);
      },
      [createConfig],
    );
    
    const handleUpdate = useCallback(
      async (id: string, req: UpdateSystemConfigRequest) => {
        await updateConfig({ id, req });
      },
      [updateConfig],
    );
    
    const handleDelete = useCallback(
      async (id: string) => {
        await deleteConfig(id);
      },
      [deleteConfig],
    );

    const handleView = (config: SystemConfig) => {
      void config;
    };

    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            {t(titleKey, {}, { fallback: t("components.systemConfig.titleFallback") })}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t(subtitleKey, {}, { fallback: t("components.systemConfig.subtitleFallback") })}
          </p>
        </div>
        <SystemConfigPanel
          configs={configs}
          categories={categories}
          activeCategory={activeCategory}
          onCategoryChange={setActiveCategory}
          onCreate={handleCreate}
          onUpdate={handleUpdate}
          onDelete={handleDelete}
          onView={handleView}
          loading={loading}
        />
      </div>
    );
  };
}
