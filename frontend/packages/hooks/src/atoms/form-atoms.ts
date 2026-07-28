/**
 * Form Atoms Factory
 *
 * Creates standardized atoms for form state management.
 */

import { atom, type PrimitiveAtom, type WritableAtom, type Atom } from "jotai";
import { atomWithStorage } from "jotai/utils";

/**
 * Form atom configuration
 */
export interface FormAtomConfig<T> {
  /** Initial form values */
  initialValue: T;
  /** Storage key for draft persistence (optional) */
  storageKey?: string;
  /** Validate function */
  validate?: (values: T) => Partial<Record<keyof T, string>>;
  /** Form name for debugging */
  name?: string;
}

/**
 * Form atoms bundle
 */
export interface FormAtoms<T> {
  /** Form values */
  values: PrimitiveAtom<T>;
  /** Form errors */
  errors: Atom<Partial<Record<keyof T, string>>>;
  /** Whether form has errors */
  hasErrors: Atom<boolean>;
  /** Whether form is dirty (modified from initial) */
  isDirty: Atom<boolean>;
  /** Set a single field value */
  setField: WritableAtom<null, [keyof T, T[keyof T]], void>;
  /** Reset form to initial values */
  reset: WritableAtom<null, [], void>;
}

/**
 * Creates a form atom with validation and state tracking
 *
 * @example
 * ```typescript
 * interface UserForm {
 *   name: string;
 *   email: string;
 * }
 *
 * const userForm = createFormAtom<UserForm>({
 *   name: "userForm",
 *   initialValue: { name: "", email: "" },
 *   validate: (values) => {
 *     const errors: Partial<Record<keyof UserForm, string>> = {};
 *     if (!values.name) errors.name = "Name is required";
 *     if (!values.email.includes("@")) errors.email = "Invalid email";
 *     return errors;
 *   },
 * });
 *
 * const [values, setValues] = useAtom(userForm.values);
 * const errors = useAtomValue(userForm.errors);
 * const [, setField] = useAtom(userForm.setField);
 * ```
 */
export function createFormAtom<T extends Record<string, unknown>>(
  config: FormAtomConfig<T>
): FormAtoms<T> {
  const { initialValue, storageKey, validate, name } = config;

  // Values atom (with optional persistence)
  const valuesAtom = storageKey
    ? atomWithStorage<T>(storageKey, initialValue)
    : atom<T>(initialValue);
  valuesAtom.debugLabel = name ? `${name}FormValuesAtom` : undefined;

  // Store initial value for dirty check
  const initialValueAtom = atom<T>(initialValue);

  // Errors atom (derived from validation)
  const errorsAtom = atom((get) => {
    if (!validate) return {} as Partial<Record<keyof T, string>>;
    return validate(get(valuesAtom));
  });
  errorsAtom.debugLabel = name ? `${name}FormErrorsAtom` : undefined;

  // Has errors (derived)
  const hasErrorsAtom = atom((get) => {
    const errors = get(errorsAtom);
    return Object.keys(errors).length > 0;
  });

  // Is dirty (derived)
  const isDirtyAtom = atom((get) => {
    const values = get(valuesAtom);
    const initial = get(initialValueAtom);
    return JSON.stringify(values) !== JSON.stringify(initial);
  });

  // Set field action
  const setFieldAtom = atom(
    null,
    (_get, set, field: keyof T, value: T[keyof T]) => {
      set(valuesAtom, (prev) => ({ ...prev, [field]: value }));
    }
  );

  // Reset action
  const resetAtom = atom(null, (_get, set) => {
    set(valuesAtom, initialValue);
  });

  return {
    values: valuesAtom as PrimitiveAtom<T>,
    errors: errorsAtom,
    hasErrors: hasErrorsAtom,
    isDirty: isDirtyAtom,
    setField: setFieldAtom,
    reset: resetAtom,
  };
}

/**
 * Creates a form draft atom (auto-saved to storage)
 *
 * @example
 * ```typescript
 * const draftForm = createFormDraftAtom<UserForm>({
 *   storageKey: "user-form-draft",
 *   initialValue: { name: "", email: "" },
 * });
 * ```
 */
export function createFormDraftAtom<T extends Record<string, unknown>>(
  config: Omit<FormAtomConfig<T>, "storageKey"> & { storageKey: string }
): FormAtoms<T> {
  return createFormAtom({
    ...config,
    storageKey: config.storageKey,
  });
}
