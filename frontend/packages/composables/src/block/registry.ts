import * as React from "react";

/**
 * Block Registry — bridges `window.aliothBlockComponents` contract into Vite modules.
 *
 * HTML prototypes populate this global; this module provides the same contract
 * for production Vite-bundled code, so `BlockSlotContent` (workspace dock),
 * `ModuleLayout`, and audit scripts all see a populated registry.
 *
 * Usage (module App.tsx):
 *   import { registerBlock, registerBlockOrder } from "@alioth/composables";
 *   registerBlock("income-waybill", WaybillPage, { name: "运单列表", icon: "FileText" });
 *   registerBlockOrder(["income-waybill", "income-receivable", ...]);
 */

export interface BlockRegistration {
  /** React component or render function */
  render: (props?: Record<string, unknown>) => React.ReactNode;
  /** Display name (untranslated — use for static pages; i18n-aware modules override via navItems) */
  name?: string;
  /** Block icon name (Lucide icon string) */
  icon?: string;
  /** Number of workflow steps (for flow blocks) */
  steps?: number;
  /** Service API codes consumed by this block */
  serviceApis?: string[];
}

// ── Global registry (single source of truth) ──

declare global {
  interface Window {
    aliothBlockComponents?: Record<string, BlockRegistration>;
    aliothBlockOrder?: string[];
  }
}

/** Ensure the global registry objects exist. */
function ensureGlobal(): void {
  window.aliothBlockComponents = window.aliothBlockComponents || {};
  window.aliothBlockOrder = window.aliothBlockOrder || [];
}

/**
 * Register a Block Capability component.
 * Idempotent — calling twice with the same id updates the registration.
 *
 * @param blockId — unique block identifier (e.g. "income-waybill")
 * @param Component — React component to render for this block
 * @param meta — optional metadata (name, icon, steps, serviceApis)
 */
export function registerBlock(
  blockId: string,
  Component: React.ComponentType<Record<string, unknown>>,
  meta?: { name?: string; icon?: string; steps?: number; serviceApis?: string[] },
): void {
  ensureGlobal();
  window.aliothBlockComponents![blockId] = {
    render: (props) => React.createElement(Component, props),
    name: meta?.name,
    icon: meta?.icon,
    steps: meta?.steps,
    serviceApis: meta?.serviceApis,
  };
}

/**
 * Set navigation order for the module's blocks.
 * Replaces the current order array with the given ids.
 */
export function registerBlockOrder(blockIds: string[]): void {
  ensureGlobal();
  window.aliothBlockOrder = blockIds;
}

/**
 * Register multiple blocks at once — convenience for module init.
 *
 * @param blocks — map of blockId → [Component, meta?]
 */
export function registerBlocks(
  blocks: Record<string, [React.ComponentType<Record<string, unknown>>, { name?: string; icon?: string; steps?: number; serviceApis?: string[] }?]>,
): void {
  for (const [blockId, [Component, meta]] of Object.entries(blocks)) {
    registerBlock(blockId, Component, meta);
  }
}

/**
 * Get a registered block component by id.
 * Returns undefined if not registered.
 */
export function getBlockComponent(blockId: string): BlockRegistration | undefined {
  return window.aliothBlockComponents?.[blockId];
}

/**
 * Get all registered block ids in nav order.
 */
export function getBlockOrder(): string[] {
  return window.aliothBlockOrder ?? [];
}

/**
 * React hook — returns all registered blocks in order.
 * Re-renders when the module re-mounts; for static blocks the registry
 * is populated once at init and doesn't change.
 */
export function useBlockRegistry(): Array<{ id: string } & BlockRegistration> {
  const reg = React.useMemo(() => {
    const order = getBlockOrder();
    const comps = window.aliothBlockComponents ?? {};
    return order
      .filter((id) => comps[id])
      .map((id) => ({ id, ...comps[id] }));
  }, []);

  // If the registry is empty on first render, try once more after a microtask
  // (covers the case where blocks register themselves asynchronously).
  const [result, setResult] = React.useState(reg);
  React.useEffect(() => {
    if (result.length > 0) return;
    const timer = setTimeout(() => {
      const order = getBlockOrder();
      const comps = window.aliothBlockComponents ?? {};
      const updated = order.filter((id) => comps[id]).map((id) => ({ id, ...comps[id] }));
      if (updated.length > 0) setResult(updated);
    }, 0);
    return () => clearTimeout(timer);
  }, [result]);

  return result;
}
