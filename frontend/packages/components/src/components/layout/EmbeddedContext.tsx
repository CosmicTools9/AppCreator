/**
 * EmbeddedContext · 微前端集成模式检测
 *
 * 当 Module 通过 single-spa 加载到 Gateway 中时，
 * `createMicroAppLifecycle` 会将 `props.embedded` 写入 `window.__ALIOTH_APP_PROPS__`。
 * ModuleLayout 通过此 Context 或 `useEmbedded()` hook 检测当前是否在集成模式下运行，
 * 以决定是否渲染 Sidebar / TopBar / WorkspaceDock 外壳。
 */

import * as React from "react";

export const EmbeddedContext = React.createContext<boolean>(false);
EmbeddedContext.displayName = "EmbeddedContext";

/**
 * 读取当前是否在 Gateway 集成模式下运行。
 *
 * 优先级：
 * 1. React Context（显式传入）
 * 2. `window.__ALIOTH_APP_PROPS__?.embedded`（由 createMicroAppLifecycle 自动设置）
 */
export function useEmbedded(): boolean {
  const fromContext = React.useContext(EmbeddedContext);
  const fromWindow =
    typeof window !== "undefined"
      ? !!(window as any).__ALIOTH_APP_PROPS__?.embedded
      : false;
  return fromContext || fromWindow;
}
