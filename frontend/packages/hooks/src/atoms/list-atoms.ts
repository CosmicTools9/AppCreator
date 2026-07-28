/**
 * List Atoms Factory
 *
 * Creates standardized atoms for list and pagination state.
 */

import { atom, type PrimitiveAtom, type WritableAtom, type Atom } from "jotai";

/**
 * List configuration
 */
export interface ListConfig<T> {
  /** Initial items */
  initialItems?: T[];
  /** List name for debugging */
  name?: string;
}

/**
 * Pagination configuration
 */
export interface PaginationConfig {
  /** Initial page (1-based) */
  initialPage?: number;
  /** Initial page size */
  initialPageSize?: number;
  /** Available page sizes */
  pageSizeOptions?: number[];
  /** Total item count (for server-side pagination) */
  totalCount?: number;
  name?: string;
}

/**
 * List atoms bundle
 */
export interface ListAtoms<T> {
  /** List items */
  items: PrimitiveAtom<T[]>;
  /** Add single item */
  addItem: WritableAtom<null, [T], void>;
  /** Remove item by predicate */
  removeItem: WritableAtom<null, [(item: T) => boolean], void>;
  /** Update item by predicate */
  updateItem: WritableAtom<null, [(item: T) => boolean, Partial<T>], void>;
  /** Replace all items */
  setItems: WritableAtom<null, [T[]], void>;
  /** Item count */
  count: Atom<number>;
  /** Whether list is empty */
  isEmpty: Atom<boolean>;
}

/**
 * Creates a list atom with common operations
 *
 * @example
 * ```typescript
 * const userList = createListAtom<User>({ name: "users" });
 *
 * const [items, setItems] = useAtom(userList.items);
 * const [, addUser] = useAtom(userList.addItem);
 * const [, removeUser] = useAtom(userList.removeItem);
 * const count = useAtomValue(userList.count);
 * ```
 */
export function createListAtom<T>(config: ListConfig<T> = {}): ListAtoms<T> {
  const { initialItems = [], name } = config;

  // Items atom
  const itemsAtom = atom<T[]>(initialItems);
  itemsAtom.debugLabel = name ? `${name}ListAtom` : undefined;

  // Add item
  const addItemAtom = atom(null, (_get, set, item: T) => {
    set(itemsAtom, (prev) => [...prev, item]);
  });

  // Remove item
  const removeItemAtom = atom(null, (_get, set, predicate: (item: T) => boolean) => {
    set(itemsAtom, (prev) => prev.filter((item) => !predicate(item)));
  });

  // Update item
  const updateItemAtom = atom(
    null,
    (_get, set, predicate: (item: T) => boolean, updates: Partial<T>) => {
      set(itemsAtom, (prev) =>
        prev.map((item) => (predicate(item) ? { ...item, ...updates } : item))
      );
    }
  );

  // Set items (replace all)
  const setItemsAtom = atom(null, (_get, set, items: T[]) => {
    set(itemsAtom, items);
  });

  // Count (derived)
  const countAtom = atom((get) => get(itemsAtom).length);

  // Is empty (derived)
  const isEmptyAtom = atom((get) => get(itemsAtom).length === 0);

  return {
    items: itemsAtom,
    addItem: addItemAtom,
    removeItem: removeItemAtom,
    updateItem: updateItemAtom,
    setItems: setItemsAtom,
    count: countAtom,
    isEmpty: isEmptyAtom,
  };
}

/**
 * Pagination atoms bundle
 */
export interface PaginationAtoms {
  /** Current page (1-based) */
  page: PrimitiveAtom<number>;
  /** Page size */
  pageSize: PrimitiveAtom<number>;
  /** Available page sizes */
  pageSizeOptions: number[];
  /** Total pages (derived from totalCount if provided) */
  totalPages: Atom<number>;
  /** Whether has next page */
  hasNextPage: Atom<boolean>;
  /** Whether has previous page */
  hasPreviousPage: Atom<boolean>;
  /** Go to next page */
  nextPage: WritableAtom<null, [], void>;
  /** Go to previous page */
  previousPage: WritableAtom<null, [], void>;
  /** Go to specific page */
  goToPage: WritableAtom<null, [number], void>;
  /** Reset to first page */
  reset: WritableAtom<null, [], void>;
  /** Set total count (server-side pagination: update after each API response) */
  setTotal: WritableAtom<null, [number], void>;
}

/**
 * Creates pagination atoms
 *
 * @example
 * ```typescript
 * const pagination = createPaginationAtom({
 *   initialPageSize: 20,
 *   totalCount: 1000,
 * });
 *
 * const [page, setPage] = useAtom(pagination.page);
 * const [pageSize] = useAtom(pagination.pageSize);
 * const totalPages = useAtomValue(pagination.totalPages);
 * const [, nextPage] = useAtom(pagination.nextPage);
 * ```
 */
export function createPaginationAtom(
  config: PaginationConfig = {}
): PaginationAtoms {
const {
    initialPage = 1,
    initialPageSize = 10,
    pageSizeOptions = [10, 20, 50, 100],
    name,
  } = config;
  const totalCountAtom = atom<number | undefined>(config.totalCount);
  // Page atom
  const pageAtom = atom(Math.max(1, initialPage));
  pageAtom.debugLabel = name ? `${name}PageAtom` : undefined;

  // Page size atom
  const pageSizeAtom = atom(initialPageSize);
  pageSizeAtom.debugLabel = name ? `${name}PageSizeAtom` : undefined;

  // Total pages (derived)
  // Total pages (derived from reactive totalCountAtom)
  const totalPagesAtom = atom((get) => {
    const tc = get(totalCountAtom);
    if (tc === undefined) return Infinity;
    return Math.ceil(tc / get(pageSizeAtom));
  });
  // Has next page (derived)
  const hasNextPageAtom = atom((get) => {
    const tc = get(totalCountAtom);
    if (tc === undefined) return true;
    return get(pageAtom) < get(totalPagesAtom);
  });

  // Has previous page (derived)
  const hasPreviousPageAtom = atom((get) => get(pageAtom) > 1);

  // Next page action
  const nextPageAtom = atom(null, (get, set) => {
    if (get(hasNextPageAtom)) {
      set(pageAtom, (prev) => prev + 1);
    }
  });

  // Previous page action
  const previousPageAtom = atom(null, (get, set) => {
    if (get(hasPreviousPageAtom)) {
      set(pageAtom, (prev) => Math.max(1, prev - 1));
    }
  });

  // Go to page action
  const goToPageAtom = atom(null, (get, set, targetPage: number) => {
    const newPage = Math.max(1, targetPage);
    const tc = get(totalCountAtom);
    if (tc !== undefined) {
      set(pageAtom, Math.min(newPage, get(totalPagesAtom)));
    } else {
      set(pageAtom, newPage);
    }
  });

  // Reset action
  const resetAtom = atom(null, (_get, set) => {
    set(pageAtom, 1);
  });
  // Set total count (server-side pagination: update after each API response)
  const setTotalAtom = atom(null, (_get, set, total: number) => {
    set(totalCountAtom, total);
  });
  setTotalAtom.debugLabel = name ? `${name}SetTotalAtom` : undefined;
  return {
    page: pageAtom,
    pageSize: pageSizeAtom,
    pageSizeOptions,
    totalPages: totalPagesAtom,
    hasNextPage: hasNextPageAtom,
    hasPreviousPage: hasPreviousPageAtom,
    nextPage: nextPageAtom,
    previousPage: previousPageAtom,
    goToPage: goToPageAtom,
    reset: resetAtom,
    setTotal: setTotalAtom,
  };
}
