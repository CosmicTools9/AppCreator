//! 微前端生命周期工厂
//!
//! 消除各模块 single-spa.tsx 中 ~100 行的重复生命周期代码。

import * as React from "react";
import { createRoot, Root } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Provider as JotaiProvider } from "jotai";
import { setApiBaseURL } from "./runtime.js";

export interface MicroAppProps {
  domElement: HTMLElement;
  domElementId: string;
  name: string;
  baseUrl?: string;
  apiBaseUrl?: string;
  authToken?: string;
  navigate?: (path: string) => void;
  /** 集成模式：由 Gateway 传递，告知模块当前处于 Gateway 集成环境 */
  embedded?: boolean;
  [key: string]: unknown;
}

export interface MicroAppLifecycle {
  bootstrap: () => Promise<void>;
  mount: (props: MicroAppProps) => Promise<void>;
  unmount: (props: MicroAppProps) => Promise<void>;
}

export interface CreateMicroAppOptions {
  /** 模块标识名（用于 CSS class 和日志前缀） */
  moduleName: string;
  /** 根 React 组件 */
  App: React.ComponentType;
  /** 自定义应用包装（如 I18nProvider + BrowserRouter） */
  renderApp?: (app: React.ReactElement, props: MicroAppProps) => React.ReactElement;
  /** QueryClient 配置 */
  queryClientOptions?: ConstructorParameters<typeof QueryClient>[0];
}

/**
 * 创建 single-spa 微前端生命周期对象
 *
 * 模块侧只需提供 App 组件和模块名，其余全部由工厂封装。
 *
 * @example
 * ```tsx
 * import { createMicroAppLifecycle } from "@alioth/api";
 * import App from "./App.js";
 * import { I18nProvider } from "@alioth/i18n";
 * import { BrowserRouter } from "react-router";
 * import zhCNDict from "./locales/zh-CN.json";
 * import enDict from "./locales/en.json";
 *
 * export const { bootstrap, mount, unmount } = createMicroAppLifecycle({
 *   moduleName: "access",
 *   App,
 *   renderApp: (app, props) => (
 *     <I18nProvider initialDictionaries={{ "zh-CN": zhCNDict, en: enDict }}>
 *       <BrowserRouter basename={props.baseUrl as string}>{app}</BrowserRouter>
 *     </I18nProvider>
 *   ),
 * });
 * ```
 */
export function createMicroAppLifecycle(
  options: CreateMicroAppOptions,
): MicroAppLifecycle {
  const { moduleName, App, renderApp, queryClientOptions } = options;
  let root: Root | null = null;
  let queryClient: QueryClient | null = null;

  return {
    async bootstrap() {
      console.log(`[${moduleName}:single-spa] Bootstrap`);
      queryClient = new QueryClient({
        defaultOptions: {
          queries: {
            retry: false,
            staleTime: 5 * 60 * 1000,
          },
        },
        ...queryClientOptions,
      });
    },

    async mount(props: MicroAppProps) {
      if (props.apiBaseUrl) {
        setApiBaseURL(props.apiBaseUrl);
      }
      // 将 props 暴露到全局，供 ModuleLayout 检测集成模式
      if (typeof window !== "undefined") {
        (window as any).__ALIOTH_APP_PROPS__ = props;
      }
      console.log(`[${moduleName}:single-spa] Mount`, props);

      const container = props.domElement;
      if (!container) {
        throw new Error(`[${moduleName}:single-spa] No DOM container provided`);
      }
      container.classList.add(`mod-${moduleName}`);

      if (!root) {
        root = createRoot(container);
      }

      const appElement = renderApp
        ? renderApp(React.createElement(App), props)
        : React.createElement(App);

      root.render(
        <React.StrictMode>
          <QueryClientProvider client={queryClient!}>
            <JotaiProvider>{appElement}</JotaiProvider>
          </QueryClientProvider>
        </React.StrictMode>,
      );
    },

    async unmount() {
      console.log(`[${moduleName}:single-spa] Unmount`);
      if (root) {
        await new Promise<void>((resolve) => {
          setTimeout(() => {
            if (root) {
              root.unmount();
              root = null;
            }
            resolve();
          }, 0);
        });
      }
      if (queryClient) {
        queryClient.clear();
        queryClient = null;
      }
    },
  };
}
