/**
 * SystemConfigPanel · 系统配置面板
 *
 * NOTE: 此组件为向后兼容 stub。原有实现已迁至 Gateway，
 * Framework 保留接口定义和最小渲染结构，确保 composables 层编译不中断。
 * 功能完善版本见 Gateway/frontend/src/components/system-config/。
 */
import * as React from "react";
import type { SystemConfigPanelProps } from "./types";

export function SystemConfigPanel({
  configs,
  categories,
  activeCategory,
  onCategoryChange,
}: SystemConfigPanelProps): React.ReactElement {
  const category = categories.find((c) => c.code === activeCategory);
  const items = configs.filter((c) => c.category === activeCategory);

  return (
    <div data-testid="system-config-panel">
      <div>
        {categories.map((cat) => (
          <button
            key={cat.code}
            type="button"
            data-active={cat.code === activeCategory ? "true" : undefined}
            onClick={() => onCategoryChange(cat.code)}
          >
            {cat.name}
          </button>
        ))}
      </div>
      <div>
        {category && <h3>{category.name}</h3>}
        {items.length === 0 && <p>No configurations</p>}
        {items.map((item) => (
          <div key={item.id}>
            <span>{item.label}</span>
            <span>{String(item.value)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

SystemConfigPanel.displayName = "SystemConfigPanel";
