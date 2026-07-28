import React, { createContext, useContext, useMemo, useCallback, useEffect, useRef } from "react";
import { useAtomValue, useSetAtom } from "jotai";
import { I18nCore } from "./core.js";
import { createLocaleAtom, createDictionaryAtom, createI18nCoreAtom } from "./atoms.js";
import type { Dictionary, I18nConfig, InterpolationParams, Locale, TranslateFunction } from "./types.js";

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

interface I18nContextValue {
  core: I18nCore;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  loadDictionary: (locale: Locale, dictionary: Dictionary) => void;
  /** 字典版本计数器，变化时触发依赖它的组件重新渲染 */
  dictVersion: number;
}

const GLOBAL_CONTEXT_KEY = "__ALIOTH_I18N_CONTEXT__";
const I18nContext: React.Context<I18nContextValue | null> =
  (typeof globalThis !== "undefined" && (globalThis as any)[GLOBAL_CONTEXT_KEY]) ||
  createContext<I18nContextValue | null>(null);
if (typeof globalThis !== "undefined") {
  (globalThis as any)[GLOBAL_CONTEXT_KEY] = I18nContext;
}

function useI18nContext(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within an <I18nProvider>");
  }
  return ctx;
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

interface I18nProviderProps {
  children: React.ReactNode;
  config?: Partial<I18nConfig>;
  /** Pre-loaded dictionaries for immediate hydration */
  initialDictionaries?: Record<Locale, Dictionary>;
}

export function I18nProvider({ children, config, initialDictionaries }: I18nProviderProps) {
  const localeAtom = useMemo(() => createLocaleAtom(config), [config?.storageKey]);
  const dictionaryAtom = useMemo(() => createDictionaryAtom(initialDictionaries), [initialDictionaries]);
  const coreAtom = useMemo(() => createI18nCoreAtom(config, initialDictionaries), [
    config?.defaultLocale,
    initialDictionaries,
  ]);

  const core = useAtomValue(coreAtom);
  const setDictionaries = useSetAtom(dictionaryAtom);
  const locale = useAtomValue(localeAtom);
  const setLocaleRaw = useSetAtom(localeAtom);
  const dictVersionRef = useRef(0); // __ALIOTH_I18N_DICT_VERSION_MARKER__
  const [dictVersion, setDictVersion] = React.useState(0);

  // Sync locale changes into I18nCore
  useEffect(() => {
    core.locale = locale;
  }, [core, locale]);

  const setLocale = useCallback(
    (value: Locale) => {
      core.locale = value;
      setLocaleRaw(value);
    },
    [core, setLocaleRaw]
  );

  const loadDictionary = useCallback(
    (loc: Locale, dictionary: Dictionary) => {
      core.loadDictionary(loc, dictionary);
      setDictionaries((prev: Map<Locale, Dictionary>) => {
        const next = new Map(prev);
        const existing = next.get(loc);
        next.set(loc, existing ? { ...existing, ...dictionary } : dictionary);
        return next;
      });
      dictVersionRef.current += 1;
      setDictVersion(dictVersionRef.current);
    },
    [core, setDictionaries]
  );

  // Hydrate initial dictionaries
  useEffect(() => {
    if (initialDictionaries) {
      const map = new Map<Locale, Dictionary>();
      for (const [loc, dict] of Object.entries(initialDictionaries)) {
        core.loadDictionary(loc, dict);
        map.set(loc, dict);
      }
      setDictionaries(map);
      dictVersionRef.current += 1;
      setDictVersion(dictVersionRef.current);
    }
  }, [core, initialDictionaries, setDictionaries]);

  const value = useMemo<I18nContextValue>(
    () => ({ core, locale, setLocale, loadDictionary, dictVersion }),
    [core, locale, setLocale, loadDictionary, dictVersion]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

export function useI18n(): {
  t: TranslateFunction;
  locale: Locale;
  setLocale: (locale: Locale) => void;
  core: I18nCore;
} {
  const { core, locale, setLocale, dictVersion } = useI18nContext();

  const t = useCallback(
    (key: string, params?: InterpolationParams, options?: { locale?: Locale; fallback?: string }) => {
      return core.t(key, params, options);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [core, dictVersion]
  );

  return { t, locale, setLocale, core };
}

/** Convenience hook when you only need the translate function */
export function useT(): TranslateFunction {
  const { t } = useI18n();
  return t;
}

/** Convenience hook for locale read/write */
export function useLocale(): {
  locale: Locale;
  setLocale: (locale: Locale) => void;
} {
  const { locale, setLocale } = useI18n();
  return { locale, setLocale };
}

/** Hook to load a dictionary dynamically (e.g. lazy-loaded module locales) */
export function useLoadDictionary(): (locale: Locale, dictionary: Dictionary) => void {
  const { loadDictionary } = useI18nContext();
  return loadDictionary;
}

// ---------------------------------------------------------------------------
// ModuleI18nShell — 统一的模块级 i18n 容器
// ---------------------------------------------------------------------------

export interface ModuleI18nShellProps {
  children: React.ReactNode;
  /** 模块专有字典（必须提供） */
  dictionaries: Record<Locale, Dictionary>;
  /** 可选的 I18nCore 配置覆盖 */
  config?: Partial<I18nConfig>;
  /** 模块已知的专有名词键（品牌名/技术术语等无需翻译的键） */
  properNouns?: string[];
}

/**
 * 模块级 i18n 容器
 *
 * 每个模块只需在 App.tsx 中用此组件包裹即可完成 i18n 接入：
 * - 自动创建 I18nProvider（兼容独立开发模式和 Gateway 微前端挂载）
 * - 通过 useLoadDictionary 动态注入模块字典
 * - 注册模块级别的专有名词白名单
 *
 * @example
 * ```tsx
 * import { ModuleI18nShell } from "@alioth/i18n";
 * import zhCN from "./locales/zh-CN.json";
 * import en from "./locales/en.json";
 *
 * export default function App() {
 *   return (
 *     <ModuleI18nShell dictionaries={{ "zh-CN": zhCN, en }}>
 *       <Routes>...</Routes>
 *     </ModuleI18nShell>
 *   );
 * }
 * ```
 */
export function ModuleI18nShell({ children, dictionaries, config, properNouns }: ModuleI18nShellProps) {
  const [booted, setBooted] = React.useState(false);

  return (
    <I18nProvider config={config} initialDictionaries={dictionaries}>
      <ModuleI18nBootLoader
        dictionaries={dictionaries}
        properNouns={properNouns}
        onBooted={() => setBooted(true)}
      />
      {booted ? children : null}
    </I18nProvider>
  );
}

/** 内部组件：通过 useLoadDictionary 动态注入 + 注册专有名词 */
function ModuleI18nBootLoader({
  dictionaries,
  properNouns,
  onBooted,
}: {
  dictionaries: Record<Locale, Dictionary>;
  properNouns?: string[];
  onBooted: () => void;
}) {
  const loadDictionary = useLoadDictionary();
  const { core } = useI18n();

  useEffect(() => {
    // 动态注入模块字典（initialDictionaries 已处理初始加载，此处确保运行时合并）
    for (const [locale, dict] of Object.entries(dictionaries)) {
      loadDictionary(locale, dict as Dictionary);
    }

    // 注册模块级专有名词
    if (properNouns && properNouns.length > 0) {
      core.properNouns.register(...properNouns);
    }

    onBooted();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return null;
}

ModuleI18nShell.displayName = "ModuleI18nShell";
