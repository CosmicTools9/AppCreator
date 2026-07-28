//! 模块独立运行入口工厂
//!
//! 消除各模块 main.tsx 中 ~35 行的重复入口代码。

import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider as JotaiProvider } from "jotai";
import { StrictMode } from "react";
import { I18nProvider } from "@alioth/i18n";
import { ThemeProvider } from "next-themes";
import type { Dictionary } from "@alioth/i18n";

declare const window: Window & { __MODULE_PREVIEW__?: true };

export interface BootstrapOptions {
  /** 模块标识名（用于 root 元素 CSS class） */
  moduleName: string;
  /** 根 React 组件 */
  App: React.ComponentType;
  /** i18n 字典：{ "zh-CN": {...}, "en": {...} } */
  dictionaries?: Record<string, Dictionary>;
  /** 自定义应用包装 */
  wrapApp?: (app: React.ReactElement) => React.ReactElement;
  /** QueryClient 配置 */
  queryClientOptions?: ConstructorParameters<typeof QueryClient>[0];
}

/**
 * 启动模块独立运行模式
 *
 * 供模块 `main.tsx` 调用，自动处理 root 元素查找、CSS class 注入和 Provider 包装。
 *
 * @example
 * ```tsx
 * import { bootstrapModule } from "@alioth/api";
 * import App from "./App.js";
 * bootstrapModule({ moduleName: "access", App });
 * ```
 */
export function bootstrapModule(options: BootstrapOptions): void {
  const { moduleName, App, dictionaries, wrapApp, queryClientOptions } = options;

  // 如果运行在 ModulePreviewApp 的预览容器中，跳过 createRoot。
  // 预览容器已通过框架组件加载 App，无需再次挂载。
  if (typeof window !== "undefined" && window.__MODULE_PREVIEW__) return;

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 60 * 1000,
        retry: 1,
      },
    },
    ...queryClientOptions,
  });

  const rootElement = document.getElementById("root");
  if (rootElement) {
    rootElement.classList.add(`mod-${moduleName}`);

    let appElement = <App />;
    if (dictionaries) {
      appElement = (
        <I18nProvider initialDictionaries={dictionaries}>
          {appElement}
        </I18nProvider>
      );
    }
    if (wrapApp) {
      appElement = wrapApp(appElement);
    }

    appElement = (
      <ThemeProvider defaultTheme="system" enableSystem>
        {appElement}
      </ThemeProvider>
    );

    createRoot(rootElement).render(
      <StrictMode>
        <QueryClientProvider client={queryClient}>
          <JotaiProvider>{appElement}</JotaiProvider>
        </QueryClientProvider>
      </StrictMode>,
    );
  }
}
