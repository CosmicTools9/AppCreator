/**
 * createEntityListPage · 实体列表页工厂
 *
 * NOTE: 此组件为向后兼容 stub。完整实现在 Gateway/frontend 中。
 * 功能完善版见 Gateway/frontend/src/pages/EntityListPage.tsx。
 */
import * as React from "react";
import type { EntityListPageConfig, ConfigurableEntityListPage } from "./types";

export const createEntityListPage: ConfigurableEntityListPage = function createEntityListPage(
  config: EntityListPageConfig,
) {
  const ListPage: React.FC = function ListPage() {
    return React.createElement("div", {
      "data-testid": "entity-list-stub",
      "data-module": config.moduleName,
      "data-domain": config.domainName,
    });
  };
  ListPage.displayName = `EntityListPage(${config.moduleName}.${config.domainName})`;
  return ListPage;
};
