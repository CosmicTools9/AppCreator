import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import type { WritableAtom } from "jotai";
import { I18nCore } from "./core.js";
import type { Dictionary, I18nConfig, Locale } from "./types.js";

const DEFAULT_STORAGE_KEY = "alioth-locale";

function createStorageKey(config?: Partial<I18nConfig>): string {
  return config?.storageKey ?? DEFAULT_STORAGE_KEY;
}

/** Create locale atom with persistent storage. Call once per app lifecycle. */
export function createLocaleAtom(
  config?: Partial<I18nConfig>
): WritableAtom<Locale, [Locale], void> {
  const defaultLocale = config?.defaultLocale ?? "zh-CN";
  const storageKey = createStorageKey(config);

  const baseAtom = atomWithStorage<Locale>(storageKey, defaultLocale, undefined, {
    getOnInit: true,
  });

  return atom(
    (get) => get(baseAtom),
    (_get, set, value: Locale) => {
      set(baseAtom, value);
    }
  );
}

/** Atom holding the loaded dictionaries per locale */
export function createDictionaryAtom(
  initialDictionaries?: Record<Locale, Dictionary>
): WritableAtom<
  Map<Locale, Dictionary>,
  [Map<Locale, Dictionary> | ((prev: Map<Locale, Dictionary>) => Map<Locale, Dictionary>)],
  void
> {
  const map = new Map<Locale, Dictionary>();
  if (initialDictionaries) {
    for (const [locale, dictionary] of Object.entries(initialDictionaries)) {
      map.set(locale as Locale, dictionary);
    }
  }
  return atom(map);
}

/** Atom factory for the I18nCore instance. Usually instantiated once at provider level. */
export function createI18nCoreAtom(
  config?: Partial<I18nConfig>,
  initialDictionaries?: Record<Locale, Dictionary>
) {
  const core = new I18nCore(config);
  if (initialDictionaries) {
    for (const [locale, dictionary] of Object.entries(initialDictionaries)) {
      core.loadDictionary(locale as Locale, dictionary);
    }
  }
  return atom<I18nCore>(core);
}

/** Re-export for consumers that want direct Jotai utilities */
export { atomWithStorage };
