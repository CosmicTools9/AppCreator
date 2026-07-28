import type { TokenGroup } from './index';

/**
 * Core Color Tokens
 *
 * Primitive color values organized by hue.
 * These map to HSL values for CSS variable compatibility.
 */
export const coreColors: TokenGroup = {
  // Primary Blue Scale
  blue: {
    50: { $value: '#eff6ff', $type: 'color', $description: 'Blue 50' },
    100: { $value: '#dbeafe', $type: 'color', $description: 'Blue 100' },
    200: { $value: '#bfdbfe', $type: 'color', $description: 'Blue 200' },
    300: { $value: '#93c5fd', $type: 'color', $description: 'Blue 300' },
    400: { $value: '#60a5fa', $type: 'color', $description: 'Blue 400' },
    500: { $value: '#3b82f6', $type: 'color', $description: 'Blue 500 - Primary' },
    600: { $value: '#2563eb', $type: 'color', $description: 'Blue 600' },
    700: { $value: '#1d4ed8', $type: 'color', $description: 'Blue 700' },
    800: { $value: '#1e40af', $type: 'color', $description: 'Blue 800' },
    900: { $value: '#1e3a8a', $type: 'color', $description: 'Blue 900' },
    950: { $value: '#172554', $type: 'color', $description: 'Blue 950' },
  },

  // Slate Scale (Neutral)
  slate: {
    50: { $value: '#f8fafc', $type: 'color', $description: 'Slate 50' },
    100: { $value: '#f1f5f9', $type: 'color', $description: 'Slate 100' },
    200: { $value: '#e2e8f0', $type: 'color', $description: 'Slate 200' },
    300: { $value: '#cbd5e1', $type: 'color', $description: 'Slate 300' },
    400: { $value: '#94a3b8', $type: 'color', $description: 'Slate 400' },
    500: { $value: '#64748b', $type: 'color', $description: 'Slate 500' },
    600: { $value: '#475569', $type: 'color', $description: 'Slate 600' },
    700: { $value: '#334155', $type: 'color', $description: 'Slate 700' },
    800: { $value: '#1e293b', $type: 'color', $description: 'Slate 800' },
    900: { $value: '#0f172a', $type: 'color', $description: 'Slate 900' },
    950: { $value: '#020617', $type: 'color', $description: 'Slate 950' },
  },

  // Semantic Colors
  red: {
    50: { $value: '#fef2f2', $type: 'color' },
    100: { $value: '#fee2e2', $type: 'color' },
    200: { $value: '#fecaca', $type: 'color' },
    300: { $value: '#fca5a5', $type: 'color' },
    400: { $value: '#f87171', $type: 'color' },
    500: { $value: '#ef4444', $type: 'color', $description: 'Destructive/Error' },
    600: { $value: '#dc2626', $type: 'color' },
    700: { $value: '#b91c1c', $type: 'color' },
    800: { $value: '#991b1b', $type: 'color' },
    900: { $value: '#7f1d1d', $type: 'color' },
    950: { $value: '#450a0a', $type: 'color' },
  },

  green: {
    50: { $value: '#f0fdf4', $type: 'color' },
    100: { $value: '#dcfce7', $type: 'color' },
    200: { $value: '#bbf7d0', $type: 'color' },
    300: { $value: '#86efac', $type: 'color' },
    400: { $value: '#4ade80', $type: 'color' },
    500: { $value: '#22c55e', $type: 'color', $description: 'Success' },
    600: { $value: '#16a34a', $type: 'color' },
    700: { $value: '#15803d', $type: 'color' },
    800: { $value: '#166534', $type: 'color' },
    900: { $value: '#14532d', $type: 'color' },
    950: { $value: '#052e16', $type: 'color' },
  },

  amber: {
    50: { $value: '#fffbeb', $type: 'color' },
    100: { $value: '#fef3c7', $type: 'color' },
    200: { $value: '#fde68a', $type: 'color' },
    300: { $value: '#fcd34d', $type: 'color' },
    400: { $value: '#fbbf24', $type: 'color' },
    500: { $value: '#f59e0b', $type: 'color', $description: 'Warning' },
    600: { $value: '#d97706', $type: 'color' },
    700: { $value: '#b45309', $type: 'color' },
    800: { $value: '#92400e', $type: 'color' },
    900: { $value: '#78350f', $type: 'color' },
    950: { $value: '#451a03', $type: 'color' },
  },
};

/**
 * Semantic Color Tokens
 *
 * Colors mapped to UI purposes. These reference core colors.
 */
export const semanticColors: TokenGroup = {
  background: {
    DEFAULT: { $value: '{slate.50}', $type: 'color' },
    primary: { $value: '{blue.500}', $type: 'color' },
    secondary: { $value: '{slate.100}', $type: 'color' },
    muted: { $value: '{slate.100}', $type: 'color' },
    accent: { $value: '{slate.100}', $type: 'color' },
    destructive: { $value: '{red.500}', $type: 'color' },
    success: { $value: '{green.500}', $type: 'color' },
    warning: { $value: '{amber.500}', $type: 'color' },
  },
  foreground: {
    DEFAULT: { $value: '{slate.900}', $type: 'color' },
    primary: { $value: '#ffffff', $type: 'color' },
    secondary: { $value: '{slate.700}', $type: 'color' },
    muted: { $value: '{slate.500}', $type: 'color' },
    accent: { $value: '{slate.900}', $type: 'color' },
    destructive: { $value: '#ffffff', $type: 'color' },
    success: { $value: '#ffffff', $type: 'color' },
    warning: { $value: '#ffffff', $type: 'color' },
  },
  border: {
    DEFAULT: { $value: '{slate.200}', $type: 'color' },
    primary: { $value: '{blue.500}', $type: 'color' },
    destructive: { $value: '{red.500}', $type: 'color' },
  },
  ring: {
    DEFAULT: { $value: '{blue.500}', $type: 'color' },
    destructive: { $value: '{red.500}', $type: 'color' },
  },
};

// ── Status Color Tokens ─────────────────────────────────────
//
// Semantic status tokens for badges, cells, and status indicators.
// These map business status names to consistent Tailwind classes,
// replacing hardcoded hex/per-module color dictionaries.
//
// Usage:
//   <span className={STATUS_COLOR_TOKENS.success.badge}>Active</span>
//
// Naming convention: {badge: "bg-* text-* border-*", dot: "bg-*"}
//   badge  = pill/badge variant (bg + text + border)
//   dot    = status dot variant (bg only)

export interface StatusColorEntry {
  /** Full badge pill classes (bg + text + border) */
  badge: string;
  /** Status dot color classes (bg only) */
  dot: string;
  /** Optional background + border for card usage */
  card?: string;
}

export const STATUS_COLOR_TOKENS: Record<string, StatusColorEntry> = {
  // Semantic tokens — mapped to Tailwind color system
  success: {
    badge: "bg-success/10 text-success border-success/20",
    dot: "bg-success",
  },
  warning: {
    badge: "bg-warning/10 text-warning border-warning/20",
    dot: "bg-warning",
  },
  danger: {
    badge: "bg-destructive/10 text-destructive border-destructive/20",
    dot: "bg-destructive",
  },
  info: {
    badge: "bg-info/10 text-info border-info/20",
    dot: "bg-info",
  },
  neutral: {
    badge: "bg-muted text-muted-foreground border-border",
    dot: "bg-muted-foreground",
  },

  // Domain-specific aliases
  active:   { badge: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/30 dark:text-green-300 dark:border-green-800", dot: "bg-green-500" },
  inactive: { badge: "bg-slate-100 text-slate-600 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700", dot: "bg-slate-400" },
  draft:    { badge: "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-800", dot: "bg-blue-500" },
  pending:  { badge: "bg-amber-100 text-amber-800 border-amber-200 dark:bg-amber-900/30 dark:text-amber-300 dark:border-amber-800", dot: "bg-amber-500" },
  archived: { badge: "bg-slate-100 text-slate-500 border-slate-200 dark:bg-slate-800 dark:text-slate-500 dark:border-slate-700", dot: "bg-slate-400" },
  locked:   { badge: "bg-slate-100 text-slate-500 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700", dot: "bg-slate-400" },
  exception:{ badge: "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/30 dark:text-red-300 dark:border-red-800", dot: "bg-red-500" },
  occupied: { badge: "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-800", dot: "bg-blue-500" },
  free:     { badge: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/30 dark:text-green-300 dark:border-green-800", dot: "bg-green-500" },
};