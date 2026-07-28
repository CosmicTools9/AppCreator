//! 模块组件注册表（Module Component Registry）
//!
//! 提供运行时跨模块组件查找机制。低层模块在 bootstrap 阶段注册共享组件，
//! 高层模块在运行时不通过编译期 import 而是通过 registry.get() 动态获取。
//!
//! 与编译期 `frontend/src/exports/` 导入互补：
//! - exports/ 用于编译期显式依赖（类型安全、Tree-shaking）
//! - ModuleComponentRegistry 用于运行时动态依赖（插件化、延迟加载）

import * as React from "react";

/** 注册表项：组件 + 元数据 */
export interface RegisteredComponent {
  component: React.ComponentType<any>;
  displayName: string;
  description?: string;
}

/** 模块组件注册表 */
export class ModuleComponentRegistry {
  private components = new Map<string, Map<string, RegisteredComponent>>();

  /**
   * 注册模块的共享组件
   * @param moduleId 模块标识（如 "product"）
   * @param name 组件名称（如 "ProductSelector"）
   * @param component React 组件
   * @param meta 可选元数据
   */
  register(
    moduleId: string,
    name: string,
    component: React.ComponentType<any>,
    meta?: { description?: string },
  ): void {
    if (!this.components.has(moduleId)) {
      this.components.set(moduleId, new Map());
    }
    const moduleMap = this.components.get(moduleId)!;
    moduleMap.set(name, {
      component,
      displayName: component.displayName || name,
      description: meta?.description,
    });
  }

  /**
   * 获取已注册的组件
   * @returns 组件或 undefined（未注册时）
   */
  get(moduleId: string, name: string): React.ComponentType<any> | undefined {
    return this.components.get(moduleId)?.get(name)?.component;
  }

  /**
   * 获取组件完整注册信息
   */
  getMeta(moduleId: string, name: string): RegisteredComponent | undefined {
    return this.components.get(moduleId)?.get(name);
  }

  /**
   * 列出某模块已注册的所有组件名
   */
  listModuleComponents(moduleId: string): string[] {
    const moduleMap = this.components.get(moduleId);
    if (!moduleMap) return [];
    return Array.from(moduleMap.keys());
  }

  /**
   * 列出所有已注册模块 ID
   */
  listModules(): string[] {
    return Array.from(this.components.keys());
  }

  /**
   * 检查某组件是否已注册
   */
  has(moduleId: string, name: string): boolean {
    return this.components.get(moduleId)?.has(name) ?? false;
  }

  /**
   * 注销模块的所有组件（卸载时调用）
   */
  unregisterModule(moduleId: string): void {
    this.components.delete(moduleId);
  }
}

/** 全局单例注册表 */
export const moduleRegistry = new ModuleComponentRegistry();

/**
 * React Hook：获取已注册的跨模块组件
 *
 * @example
 * ```tsx
 * const ProductSelector = useModuleComponent("product", "ProductSelector");
 * if (!ProductSelector) return <Fallback />;
 * return <ProductSelector products={products} />;
 * ```
 */
export function useModuleComponent(
  moduleId: string,
  name: string,
): React.ComponentType<any> | undefined {
  // 使用 useSyncExternalStore 或简单引用，避免不必要的重渲染
  return React.useMemo(
    () => moduleRegistry.get(moduleId, name),
    [moduleId, name],
  );
}
