export type Locale = "zh-CN" | "en" | string;

export type Dictionary = Record<string, unknown>;

export type InterpolationParams = Record<string, string | number | Date>;

export interface I18nConfig {
  /** Default locale when none is set or found */
  defaultLocale: Locale;
  /** Supported locales */
  supportedLocales: Locale[];
  /** Initial dictionary for the default locale */
  defaultDictionary?: Dictionary;
  /** Storage key for persisting locale preference */
  storageKey?: string;
}

export interface I18nCoreState {
  locale: Locale;
  dictionaries: Map<Locale, Dictionary>;
  config: I18nConfig;
}

export type TranslateFunction = (
  key: string,
  params?: InterpolationParams,
  options?: { locale?: Locale; fallback?: string }
) => string;

/**
 * 类型安全的翻译函数
 *
 * 当 TKey 为具体字面量联合类型时，提供键名自动补全和校验；
 * 当 TKey 为 string 时，回退到普通翻译行为以兼容动态键。
 *
 * 各应用可通过 module augmentation 覆盖 useT() / useI18n() 的返回类型：
 *
 * ```ts
 * declare module "@alioth/i18n" {
 *   export function useT(): TypedTranslateFunction<MyKeys>;
 *   export function useI18n(): { t: TypedTranslateFunction<MyKeys>; ... };
 * }
 * ```
 */
export interface TypedTranslateFunction<TKey extends string = string> {
  (key: TKey, params?: InterpolationParams, options?: { locale?: Locale; fallback?: string }): string;
  (key: string, params?: InterpolationParams, options?: { locale?: Locale; fallback?: string }): string;
}

/**
 * 专有名词集 — 在 zh-CN/en 中保持相同值的键清单
 *
 * 品牌名、标准缩写、技术术语等跨语言通用符号不应被翻译。
 * 当 i18n 校验报告 "zh/en 值相同" 时，可先核对此集。
 */
export interface ProperNounSet {
  /** 已知不翻译的键集合 */
  knownKeys: ReadonlySet<string>;
  /** 注册一批专有名词键（通常由各应用/模块初始化时调用） */
  register(...keys: string[]): void;
}
