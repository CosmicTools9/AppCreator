
/**
 * Entity List page types — CRUD 工厂配置类型
 *
 * NOTE: 完整实现在 Gateway 迁移过程中拆分。
 * 当前为向后兼容 stub，保证 composables 层编译通过。
 */

import type React from "react";

export interface EntityListPageConfig {
  moduleName: string;
  domainName: string;
  listEndpoint?: string;
  createEndpoint?: string;
  updateEndpoint?: string;
  deleteEndpoint?: string;
  viewEndpoint?: string;
  pageSize?: number;
  titleKey?: string;
  subtitleKey?: string;
}

export interface EntityListPageHooks {
  beforeCreate?: (data: unknown) => unknown | Promise<unknown>;
  afterCreate?: (result: unknown) => void;
  beforeUpdate?: (data: unknown) => unknown | Promise<unknown>;
  afterUpdate?: (result: unknown) => void;
  beforeDelete?: (id: string) => boolean | Promise<boolean>;
  afterDelete?: (id: string) => void;
}

export interface EntityListTabConfig {
  key: string;
  labelKey: string;
  config: EntityListPageConfig;
}

export interface EntityTabbedListPageConfig {
  moduleName: string;
  tabs: EntityListTabConfig[];
  defaultTab?: string;
}

export interface InlineEditingConfig {
  enabled: boolean;
  saveOnBlur?: boolean;
  validateOnSave?: boolean;
}

export interface TabbedInlineEditingConfig {
  enabled: boolean;
  tabs: Record<string, InlineEditingConfig>;
}

export interface ConfigurableEntityListPage {
  (config: EntityListPageConfig): React.FC;
}

export interface ConfigurableEntityTabbedListPage {
  (config: EntityTabbedListPageConfig): React.FC;
}
