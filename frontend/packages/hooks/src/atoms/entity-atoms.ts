/**
 * Entity Atoms Factory
 *
 * Creates standardized atoms for entity selection and management.
 * Follows the pattern used in meta-admin for consistent state handling.
 */

import { atom, type PrimitiveAtom, type Atom } from "jotai";

/**
 * Configuration for entity atom creation
 */
export interface EntityAtomConfig<T> {
  /** Initial value for the entity */
  initialValue?: T | null;
  /** Entity name for debugging */
  name?: string;
}

/**
 * Entity atoms bundle
 */
export interface EntityAtoms<T> {
  /** Primary entity atom */
  entity: PrimitiveAtom<T | null>;
  /** Whether an entity is selected */
  hasEntity: Atom<boolean>;
  /** Entity ID (if T has id property) */
  entityId: Atom<string | number | undefined>;
}

/**
 * Creates a standardized entity atom with derived states
 *
 * @example
 * ```typescript
 * interface User { id: string; name: string; }
 * const userAtoms = createEntityAtom<User>("user");
 *
 * // Use in component
 * const [user, setUser] = useAtom(userAtoms.entity);
 * const hasUser = useAtomValue(userAtoms.hasEntity);
 * ```
 */
export function createEntityAtom<T extends { id?: string | number }>(
  config: EntityAtomConfig<T> = {},
): EntityAtoms<T> {
  const { initialValue = null, name } = config;

  // Primary entity atom
  const entityAtom = atom<T | null>(initialValue);
  entityAtom.debugLabel = name ? `${name}Atom` : undefined;

  // Derived: has entity
  const hasEntityAtom = atom((get) => !!get(entityAtom));
  hasEntityAtom.debugLabel = name ? `has${capitalize(name)}Atom` : undefined;

  // Derived: entity ID
  const entityIdAtom = atom((get) => get(entityAtom)?.id);
  entityIdAtom.debugLabel = name ? `${name}IdAtom` : undefined;

  return {
    entity: entityAtom,
    hasEntity: hasEntityAtom,
    entityId: entityIdAtom,
  };
}

/**
 * Creates multiple related entity atoms
 *
 * @example
 * ```typescript
 * const { user, profile, settings } = createDerivedEntityAtoms({
 *   user: { initialValue: null },
 *   profile: { initialValue: null },
 * });
 * ```
 */
export function createDerivedEntityAtoms<
  T extends Record<string, { id?: string | number }>,
>(configs: { [K in keyof T]: EntityAtomConfig<T[K]> }): {
  [K in keyof T]: EntityAtoms<T[K]>;
} {
  const result = {} as { [K in keyof T]: EntityAtoms<T[K]> };

  for (const key of Object.keys(configs) as Array<keyof T>) {
    result[key] = createEntityAtom<T[typeof key]>({
      ...configs[key],
      name: String(key),
    });
  }

  return result;
}

// Helper
function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
