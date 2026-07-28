/**
 * Modal Atoms Factory
 *
 * Creates standardized atoms for modal and dialog state management.
 */

import { atom, type PrimitiveAtom, type WritableAtom } from "jotai";

/**
 * Modal configuration
 */
export interface ModalConfig {
  /** Initial open state */
  initialOpen?: boolean;
  /** Modal name for debugging */
  name?: string;
}

/**
 * Modal group configuration
 */
export interface ModalGroupConfig {
  /** Modal names in the group */
  modals: string[];
  /** Allow multiple modals open simultaneously */
  allowMultiple?: boolean;
}

/**
 * Modal atoms bundle
 */
export interface ModalAtoms {
  /** Open state atom */
  isOpen: PrimitiveAtom<boolean>;
  /** Open action */
  open: WritableAtom<null, [], void>;
  /** Close action */
  close: WritableAtom<null, [], void>;
  /** Toggle action */
  toggle: WritableAtom<null, [], void>;
}

/**
 * Creates a standardized modal atom with actions
 *
 * @example
 * ```typescript
 * const userModal = createModalAtom({ name: "user" });
 *
 * // Use in component
 * const [isOpen, setIsOpen] = useAtom(userModal.isOpen);
 * const [, open] = useAtom(userModal.open);
 * const [, close] = useAtom(userModal.close);
 * ```
 */
export function createModalAtom(config: ModalConfig = {}): ModalAtoms {
  const { initialOpen = false, name } = config;

  // Open state
  const isOpenAtom = atom(initialOpen);
  isOpenAtom.debugLabel = name ? `${name}ModalOpenAtom` : undefined;

  // Actions
  const openAtom = atom(null, (_get, set) => {
    set(isOpenAtom, true);
  });
  openAtom.debugLabel = name ? `${name}ModalOpenAction` : undefined;

  const closeAtom = atom(null, (_get, set) => {
    set(isOpenAtom, false);
  });
  closeAtom.debugLabel = name ? `${name}ModalCloseAction` : undefined;

  const toggleAtom = atom(null, (get, set) => {
    set(isOpenAtom, !get(isOpenAtom));
  });
  toggleAtom.debugLabel = name ? `${name}ModalToggleAction` : undefined;

  return {
    isOpen: isOpenAtom,
    open: openAtom,
    close: closeAtom,
    toggle: toggleAtom,
  };
}

/**
 * Creates a group of related modals with mutual exclusion
 *
 * @example
 * ```typescript
 * const modals = createModalGroupAtom({
 *   modals: ["create", "edit", "delete"],
 *   allowMultiple: false,
 * });
 *
 * // Open create modal (closes others)
 * const [, openCreate] = useAtom(modals.create.open);
 * ```
 */
export function createModalGroupAtom<T extends string>(
  config: ModalGroupConfig & { modals: T[] }
): Record<T, ModalAtoms> & {
  /** Close all modals in the group */
  closeAll: WritableAtom<null, [], void>;
  /** Currently open modal IDs */
  openModals: ReturnType<typeof atom<string[]>>;
} {
  const { modals, allowMultiple = false } = config;
  const result = {} as Record<T, ModalAtoms>;
  const openModalsAtom = atom<string[]>([]);

  for (const modalName of modals) {
    const isOpenAtom = atom(false);
    isOpenAtom.debugLabel = `${modalName}ModalOpenAtom`;

    const openAtom = atom(null, (get, set) => {
      if (!allowMultiple) {
        // Close all other modals
        for (const other of modals) {
          if (other !== modalName) {
            set(result[other].isOpen, false);
          }
        }
        set(openModalsAtom, [modalName]);
      } else {
        set(openModalsAtom, [...get(openModalsAtom), modalName]);
      }
      set(isOpenAtom, true);
    });
    openAtom.debugLabel = `${modalName}ModalOpenAction`;

    const closeAtom = atom(null, (_get, set) => {
      set(isOpenAtom, false);
      set(
        openModalsAtom,
        (prev) => prev.filter((m) => m !== modalName)
      );
    });
    closeAtom.debugLabel = `${modalName}ModalCloseAction`;

    const toggleAtom = atom(null, (get, set) => {
      set(isOpenAtom, !get(isOpenAtom));
    });
    toggleAtom.debugLabel = `${modalName}ModalToggleAction`;

    result[modalName] = {
      isOpen: isOpenAtom,
      open: openAtom,
      close: closeAtom,
      toggle: toggleAtom,
    };
  }

  const closeAllAtom = atom(null, (_get, set) => {
    for (const modalName of modals) {
      set(result[modalName].isOpen, false);
    }
    set(openModalsAtom, []);
  });
  closeAllAtom.debugLabel = "closeAllModalsAction";

  return {
    ...result,
    closeAll: closeAllAtom,
    openModals: openModalsAtom,
  };
}
