/**
 * AliothStudio Jotai Atoms
 *
 * Standardized atom patterns for AliothStudio modules.
 * Provides reusable atom factories and common state patterns.
 *
 * Architecture:
 * - createEntityAtom: Factory for entity selection atoms
 * - createFilterAtom: Factory for filter state atoms
 * - createModalAtom: Factory for modal/dialog state atoms
 * - createFormAtom: Factory for form draft state atoms
 * - Persistence: Use atomWithStorage for persistent state
 *
 * @example
 * ```typescript
 * import { createEntityAtom, createFilterAtom } from "@alioth/hooks/atoms";
 *
 * const userAtom = createEntityAtom<User>("user");
 * const userFilterAtom = createFilterAtom<UserFilter>("user", { status: "" });
 * ```
 */

// ============================================
// Core Atom Factories
// ============================================

export {
  createEntityAtom,
  createDerivedEntityAtoms,
  type EntityAtoms,
} from "./entity-atoms";

export {
  createFilterAtom,
  createSearchAtom,
  type FilterConfig,
} from "./filter-atoms";

export {
  createModalAtom,
  createModalGroupAtom,
  type ModalConfig,
  type ModalGroupConfig,
} from "./modal-atoms";

export {
  createFormAtom,
  createFormDraftAtom,
  type FormAtomConfig,
} from "./form-atoms";

export {
  createListAtom,
  createPaginationAtom,
  type ListConfig,
  type PaginationConfig,
} from "./list-atoms";

// ============================================
// UI State Atoms
// ============================================

export {
  sidebarAtom,
  themeAtom,
  breakpointAtom,
  type Theme,
  type Breakpoint,
} from "./ui-atoms";

// ============================================
// Utility Atoms
// ============================================

export {
  atomWithLocalStorage,
  atomWithSessionStorage,
  createAsyncAtom,
  type AsyncAtomConfig,
} from "./utils";
