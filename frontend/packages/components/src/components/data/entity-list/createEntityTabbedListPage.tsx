/**
 * createEntityTabbedListPage · 标签式实体列表页工厂
 *
 * NOTE: 此组件为向后兼容 stub。完整实现在 Gateway/frontend 中。
 */
import * as React from "react";
import type { EntityTabbedListPageConfig, ConfigurableEntityTabbedListPage } from "./types";

export const createEntityTabbedListPage: ConfigurableEntityTabbedListPage =
  function createEntityTabbedListPage(config: EntityTabbedListPageConfig) {
    const TabbedListPage: React.FC = function TabbedListPage() {
      return React.createElement("div", {
        "data-testid": "entity-tabbed-list-stub",
        "data-module": config.moduleName,
        "data-tabs": config.tabs.length,
      });
    };
    TabbedListPage.displayName = `EntityTabbedListPage(${config.moduleName})`;
    return TabbedListPage;
  };
