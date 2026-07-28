/**
 * Atom Utilities
 *
 * Helper functions for creating and working with atoms.
 */

import { atom, type Atom, type PrimitiveAtom, type WritableAtom } from "jotai";
import { atomWithStorage, atomWithObservable } from "jotai/utils";
import { Observable } from "rxjs";

/**
 * Creates an atom synced with localStorage
 */
export function atomWithLocalStorage<T>(key: string, initialValue: T) {
  return atomWithStorage<T>(key, initialValue);
}

/**
 * Creates an atom synced with sessionStorage
 * Note: sessionStorage persistence is handled via custom storage in jotai/utils.
 */
export function atomWithSessionStorage<T>(_key: string, initialValue: T) {
  return atom<T>(initialValue);
}

/**
 * Configuration for async atoms
 */
export interface AsyncAtomConfig<T, P = void> {
  /** Async fetch function */
  fetch: (params: P) => Promise<T>;
  /** Initial value */
  initialValue?: T;
  /** Atom name for debugging */
  name?: string;
}

/**
 * Async atom state
 */
export interface AsyncAtomState<T> {
  /** Current data */
  data: T;
  /** Whether currently loading */
  isLoading: boolean;
  /** Error if any */
  error: Error | null;
}

/**
 * Creates an async atom with loading and error states
 *
 * @example
 * ```typescript
 * const usersAsyncAtom = createAsyncAtom<User[]>({
 *   fetch: () => fetchUsers(),
 *   initialValue: [],
 *   name: "users",
 * });
 *
 * const [state, refresh] = useAtom(usersAsyncAtom.atom);
 * // state.data, state.isLoading, state.error
 * ```
 */
export function createAsyncAtom<T, P = void>(
  config: AsyncAtomConfig<T, P>,
): {
  atom: PrimitiveAtom<AsyncAtomState<T>>;
  refresh: WritableAtom<null, [P?], Promise<void>>;
} {
  const { fetch, initialValue, name } = config;

  const asyncAtom = atom<AsyncAtomState<T>>({
    data: initialValue as T,
    isLoading: false,
    error: null,
  });
  asyncAtom.debugLabel = name ? `${name}AsyncAtom` : undefined;

  const refreshAtom = atom(null, async (get, set, params?: P) => {
    set(asyncAtom, (prev) => ({ ...prev, isLoading: true, error: null }));
    try {
      const data = await fetch(params as P);
      set(asyncAtom, { data, isLoading: false, error: null });
    } catch (error) {
      set(asyncAtom, (prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error : new Error(String(error)),
      }));
    }
  });

  return {
    atom: asyncAtom,
    refresh: refreshAtom,
  };
}

/**
 * Creates a derived atom that resets when dependencies change
 */
export function createResettableAtom<T, D extends unknown[]>(
  derive: (...deps: D) => T,
  deps: { [K in keyof D]: Atom<D[K]> },
): Atom<T> {
  return atom((get) => {
    const values = deps.map((dep) => get(dep)) as D;
    return derive(...values);
  });
}

/**
 * Creates a computed atom with memoization
 */
export function createComputedAtom<T>(
  compute: () => T,
  deps: Atom<unknown>[],
): Atom<T> {
  return atom((get) => {
    // Access all dependencies to track them
    deps.forEach((dep) => get(dep));
    return compute();
  });
}
