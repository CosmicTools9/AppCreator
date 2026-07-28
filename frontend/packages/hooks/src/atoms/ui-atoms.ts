/**
 * UI Atoms
 *
 * Common UI state atoms for AliothStudio modules.
 */

import { atom, type PrimitiveAtom } from "jotai";
import { atomWithStorage } from "jotai/utils";

// ============================================
// Theme
// ============================================

export type Theme = "light" | "dark" | "system";

/**
 * Theme atom - persisted to localStorage
 */
export const themeAtom = atomWithStorage<Theme>("alioth-theme", "system");

// ============================================
// Sidebar
// ============================================

export interface SidebarState {
  /** Whether sidebar is expanded */
  isOpen: boolean;
  /** Currently active item ID */
  activeItemId: string | null;
  /** Expanded section IDs */
  expandedSections: string[];
}

/**
 * Sidebar atom - persisted to localStorage
 */
export const sidebarAtom = atomWithStorage<SidebarState>("alioth-sidebar", {
  isOpen: true,
  activeItemId: null,
  expandedSections: [],
});

// ============================================
// Breakpoint
// ============================================

export type Breakpoint = "mobile" | "tablet" | "desktop" | "wide";

/**
 * Breakpoint atom - reflects current viewport size
 * Should be updated by a hook listening to window resize
 */
export const breakpointAtom = atom<Breakpoint>("desktop");

/**
 * Mobile detection atom (derived)
 */
export const isMobileAtom = atom((get) => {
  const bp = get(breakpointAtom);
  return bp === "mobile";
});

/**
 * Desktop detection atom (derived)
 */
export const isDesktopAtom = atom((get) => {
  const bp = get(breakpointAtom);
  return bp === "desktop" || bp === "wide";
});
