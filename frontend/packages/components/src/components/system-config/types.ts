/**
 * System Config types · 系统配置面板类型定义
 *
 * 这些类型用于 `createSystemConfigPage` 工厂函数和 `SystemConfigPanel` 组件。
 * 实现代码于重构时移至 Gateway，Framework 保留类型定义供 composables 层使用。
 */

export interface ConfigCategory {
  code: string;
  name: string;
  description?: string;
}

export type ConfigCategoryCode = string;

export interface SystemConfig {
  id: string;
  key: string;
  value: unknown;
  category: ConfigCategoryCode;
  label: string;
  description?: string;
  type: "text" | "number" | "boolean" | "select" | "json";
  options?: { label: string; value: string }[];
  order?: number;
}

export interface CreateSystemConfigRequest {
  key: string;
  value: unknown;
  category: ConfigCategoryCode;
  label: string;
  type: SystemConfig["type"];
}

export interface UpdateSystemConfigRequest {
  value: unknown;
  label?: string;
}

export interface SystemConfigPanelProps {
  configs: SystemConfig[];
  categories: ConfigCategory[];
  activeCategory: string;
  onCategoryChange: (category: string) => void;
  onCreate: (config: CreateSystemConfigRequest) => void;
  onUpdate: (id: string, config: UpdateSystemConfigRequest) => void;
  onDelete: (id: string) => void;
  onView?: (config: SystemConfig) => void;
  loading?: boolean;
}
