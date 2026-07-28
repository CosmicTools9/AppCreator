/**
 * Filter Atoms Factory
 *
 * Creates standardized atoms for filtering and search state.
 */

import { atom, type PrimitiveAtom, type Atom } from "jotai";
import { atomWithStorage } from "jotai/utils";

/**
 * Filter configuration
 */
export interface FilterConfig<T> {
  /** Storage key for persistence (if undefined, not persisted) */
  storageKey?: string;
  /** Initial filter values */
  initialValue: T;
  /** Filter name for debugging */
  name?: string;
}

/**
 * Search atom configuration
 */
export interface SearchConfig {
  /** Storage key for persistence */
  storageKey?: string;
  /** Initial search query */
  initialValue?: string;
  /** Debounce delay in ms */
  debounceMs?: number;
  name?: string;
}

/**
 * Creates a filter atom with optional persistence
 *
 * @example
 * ```typescript
 * interface UserFilter {
 *   status: string;
 *   role: string;
 * }
 *
 * const filterAtom = createFilterAtom<UserFilter>({
 *   initialValue: { status: "", role: "" },
 * });
 *
 * // Persisted version
 * const persistedFilterAtom = createFilterAtom<UserFilter>({
 *   storageKey: "user-filter",
 *   initialValue: { status: "", role: "" },
 * });
 * ```
 */
export function createFilterAtom<T extends Record<string, unknown>>(
  config: FilterConfig<T>
): PrimitiveAtom<T> {
  const { storageKey, initialValue, name } = config;

  const filterAtom = storageKey
    ? atomWithStorage<T>(storageKey, initialValue)
    : atom<T>(initialValue);

  if (name) {
    filterAtom.debugLabel = `${name}FilterAtom`;
  }

  return filterAtom as PrimitiveAtom<T>;
}

/**
 * Creates a search atom with normalized query
 *
 * @example
 * ```typescript
 * const searchAtoms = createSearchAtom({
 *   storageKey: "user-search",
 *   debounceMs: 300,
 * });
 *
 * const [query, setQuery] = useAtom(searchAtoms.query);
 * const normalized = useAtomValue(searchAtoms.normalized);
 * ```
 */
export function createSearchAtom(
  config: SearchConfig = {}
): {
  query: PrimitiveAtom<string>;
  normalized: Atom<string>;
  isEmpty: Atom<boolean>;
} {
  const {
    storageKey,
    initialValue = "",
    name = "search",
  } = config;

  // Query atom (with optional persistence)
  const queryAtom = storageKey
    ? atomWithStorage<string>(storageKey, initialValue)
    : atom<string>(initialValue);
  queryAtom.debugLabel = `${name}QueryAtom`;

  // Normalized query (lowercase, trimmed)
  const normalizedAtom = atom((get) =>
    get(queryAtom).trim().toLowerCase()
  );
  normalizedAtom.debugLabel = `${name}NormalizedQueryAtom`;

  // Is empty check
  const isEmptyAtom = atom((get) => get(normalizedAtom).length === 0);
  isEmptyAtom.debugLabel = `${name}IsEmptyAtom`;

  return {
    query: queryAtom as PrimitiveAtom<string>,
    normalized: normalizedAtom,
    isEmpty: isEmptyAtom,
  };
}
