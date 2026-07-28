// Module identity probe: helps diagnose duplicate module instances in micro-frontend dev mode
if (typeof window !== "undefined") {
  const loadedFrom = (globalThis as any).__ALIOTH_I18N_LOADED_URL__;
  const myUrl = typeof import.meta.url !== "undefined" ? import.meta.url : "unknown";
  if (loadedFrom) {
    console.warn(`[@alioth/i18n] DUPLICATE MODULE INSTANCE DETECTED! Previous: ${loadedFrom}, Current: ${myUrl}`);
  } else {
    console.log(`[@alioth/i18n] Module loaded from: ${myUrl}`);
  }
  (globalThis as any).__ALIOTH_I18N_LOADED_URL__ = myUrl;
}

// Core
export { I18nCore } from "./core.js";
export type {
  Locale,
  Dictionary,
  InterpolationParams,
  I18nConfig,
  I18nCoreState,
  TranslateFunction,
  TypedTranslateFunction,
  ProperNounSet,
} from "./types.js";

// Atoms
export {
  createLocaleAtom,
  createDictionaryAtom,
  createI18nCoreAtom,
  atomWithStorage,
} from "./atoms.js";

// React
export {
  I18nProvider,
  ModuleI18nShell,
  useI18n,
  useT,
  useLocale,
  useLoadDictionary,
} from "./react.js";
export type { ModuleI18nShellProps } from "./react";
