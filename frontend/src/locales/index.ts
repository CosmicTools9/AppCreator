import { useCallback } from "react";
import zhCN from "./zh-CN.json";
import en from "./en.json";

const LOCALES: Record<string, Record<string, any>> = { "zh-CN": zhCN, en };

type Lang = keyof typeof LOCALES;

function get(lang: Lang, key: string): string {
  const keys = key.split(".");
  let obj: any = LOCALES[lang];
  for (const k of keys) {
    if (obj == null || typeof obj !== "object") return key;
    obj = obj[k];
  }
  return typeof obj === "string" ? obj : key;
}

const FALLBACK_LANG: Lang =
  typeof navigator !== "undefined"
    ? (navigator.language.startsWith("zh") ? "zh-CN" : "en")
    : "zh-CN";

/**
 * Minimal i18n hook. Returns `t()` for dot-path key lookup.
 * Auto-detects browser language; falls back to zh-CN.
 */
export function useT() {
  const lang: Lang = FALLBACK_LANG;

  const t = useCallback(
    (key: string, vars?: Record<string, string | number>) => {
      let val = get(lang, key);
      if (vars) {
        for (const [k, v] of Object.entries(vars)) {
          val = val.replace(`{${k}}`, String(v));
        }
      }
      return val;
    },
    [lang]
  );

  return { t, lang };
}
