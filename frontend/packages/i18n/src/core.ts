import type { Dictionary, InterpolationParams, I18nConfig, Locale, ProperNounSet } from "./types";

const DEFAULT_CONFIG: I18nConfig = {
  defaultLocale: "zh-CN",
  supportedLocales: ["zh-CN", "en"],
  storageKey: "alioth-locale",
};

function getNestedValue(obj: Dictionary, path: string): string | undefined {
  // First try exact key match (flat dictionary keys like "app.title")
  const exact = (obj as Record<string, unknown>)[path];
  if (typeof exact === "string") return exact;
  if (typeof exact === "number") return String(exact);

  // Then try nested path traversal (nested objects like { app: { title: "..." } })
  const keys = path.split(".");
  let current: unknown = obj;
  for (const key of keys) {
    if (current === null || current === undefined) return undefined;
    if (typeof current !== "object") return undefined;
    current = (current as Record<string, unknown>)[key];
  }
  if (typeof current === "string") return current;
  if (typeof current === "number") return String(current);
  return undefined;
}

function interpolate(template: string, params: InterpolationParams): string {
  return template.replace(/\{(\w+)\}/g, (_match, key) => {
    const value = params[key];
    if (value instanceof Date) {
      // Default date formatting — callers should pre-format for specific patterns
      return value.toLocaleDateString();
    }
    if (value === undefined || value === null) {
      return `{${key}}`;
    }
    return String(value);
  });
}

// ---------------------------------------------------------------------------
// Proper Noun Set
// ---------------------------------------------------------------------------

/** 创建专有名词集合 — 品牌名/标准缩写/技术术语等不应被翻译的键 */
export function createProperNounSet(initialKeys: string[] = []): ProperNounSet {
  const keys = new Set(initialKeys);
  return {
    knownKeys: keys,
    register(...newKeys: string[]) {
      for (const k of newKeys) keys.add(k);
    },
  };
}

// ---------------------------------------------------------------------------
// I18nCore
// ---------------------------------------------------------------------------

export class I18nCore {
  private config: I18nConfig;
  private dictionaries: Map<Locale, Dictionary> = new Map();
  private currentLocale: Locale;
  /** 全局专有名词集 — 通过 ModuleI18nShell / main.tsx 初始化 */
  properNouns: ProperNounSet = createProperNounSet();

  constructor(config: Partial<I18nConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.currentLocale = this.resolveInitialLocale();
    if (this.config.defaultDictionary) {
      this.dictionaries.set(this.config.defaultLocale, this.config.defaultDictionary);
    }
  }

  get locale(): Locale {
    return this.currentLocale;
  }

  set locale(value: Locale) {
    this.currentLocale = value;
    this.persistLocale(value);
  }

  get fallbackLocale(): Locale {
    return this.config.defaultLocale;
  }

  loadDictionary(locale: Locale, dictionary: Dictionary): void {
    const existing = this.dictionaries.get(locale);
    if (existing) {
      this.dictionaries.set(locale, { ...existing, ...dictionary });
    } else {
      this.dictionaries.set(locale, dictionary);
    }
  }

  unloadDictionary(locale: Locale): void {
    this.dictionaries.delete(locale);
  }

  t(
    key: string,
    params?: InterpolationParams,
    options?: { locale?: Locale; fallback?: string }
  ): string {
    const targetLocale = options?.locale ?? this.currentLocale;
    const raw = this.lookup(key, targetLocale);
    if (raw !== undefined) {
      return params ? interpolate(raw, params) : raw;
    }
    // Fallback chain: target -> defaultLocale -> key itself
    if (targetLocale !== this.config.defaultLocale) {
      const fallbackRaw = this.lookup(key, this.config.defaultLocale);
      if (fallbackRaw !== undefined) {
        return params ? interpolate(fallbackRaw, params) : fallbackRaw;
      }
    }
    return options?.fallback ?? key;
  }

  formatDate(
    value: Date | number,
    options?: Intl.DateTimeFormatOptions,
    locale?: Locale
  ): string {
    const d = typeof value === "number" ? new Date(value) : value;
    return new Intl.DateTimeFormat(locale ?? this.currentLocale, options).format(d);
  }

  formatNumber(
    value: number,
    options?: Intl.NumberFormatOptions,
    locale?: Locale
  ): string {
    return new Intl.NumberFormat(locale ?? this.currentLocale, options).format(value);
  }

  formatCurrency(
    value: number,
    currency: string,
    locale?: Locale
  ): string {
    return new Intl.NumberFormat(locale ?? this.currentLocale, {
      style: "currency",
      currency,
    }).format(value);
  }

  formatRelativeTime(
    value: number,
    unit: Intl.RelativeTimeFormatUnit,
    locale?: Locale
  ): string {
    return new Intl.RelativeTimeFormat(locale ?? this.currentLocale, {
      numeric: "auto",
    }).format(value, unit);
  }

  private lookup(key: string, locale: Locale): string | undefined {
    const dict = this.dictionaries.get(locale);
    if (!dict) {
      if (typeof window !== "undefined") {
        console.warn(`[i18n] No dictionary for locale "${locale}". Available:`, [...this.dictionaries.keys()]);
      }
      return undefined;
    }
    const value = getNestedValue(dict, key);
    if (value === undefined && typeof window !== "undefined") {
      console.warn(`[i18n] Key not found: "${key}" in locale "${locale}". Dict keys sample:`, Object.keys(dict).slice(0, 20));
    }
    return value;
  }

  private resolveInitialLocale(): Locale {
    if (typeof window !== "undefined" && window.localStorage) {
      try {
        const stored = window.localStorage.getItem(this.config.storageKey!);
        if (stored && this.config.supportedLocales.includes(stored)) {
          return stored;
        }
      } catch {
        /* ignore storage errors */
      }
    }
    return this.config.defaultLocale;
  }

  private persistLocale(value: Locale): void {
    if (typeof window !== "undefined" && window.localStorage) {
      try {
        window.localStorage.setItem(this.config.storageKey!, value);
      } catch {
        /* ignore storage errors */
      }
    }
  }
}
