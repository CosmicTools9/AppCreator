/**
 * createBlockRoutes — auto-generate React Router `<Route>` elements from block components.
 *
 * Bridges the `window.aliothBlockComponents` contract into Vite-module routing.
 * Instead of hand-writing N `<Route>` elements in each module's `App.tsx`,
 * modules define a `blockComponents` map and call this function.
 *
 * Also populates the global aliothBlockComponents registry (for workspace dock
 * and audit script compatibility) and aliothBlockOrder (for nav order).
 *
 * @example
 * ```tsx
 * import { createBlockRoutes } from "@alioth/composables";
 * import { Route } from "react-router";
 *
 * const blockComponents = {
 *   "income-waybill": WaybillPage,
 *   "income-receivable": ReceivablePage,
 * };
 * const blockRoutes = createBlockRoutes(blockComponents, navItems);
 *
 * <Routes>
 *   <Route element={<ModuleLayout />}>
 *     {blockRoutes}
 *   </Route>
 * </Routes>
 * ```
 */

import * as React from 'react';
import { Route, Navigate } from 'react-router';
import type { MainNavItem } from '@alioth/components';
import { registerBlock, registerBlockOrder } from './registry';

/** Metadata for a block route entry */
export interface BlockRouteMeta {
  /** Display name (shown in nav) */
  name?: string;
  /** Lucide icon name */
  icon?: string;
}

/** Map of block ID → React component */
export type BlockComponentMap = Record<string, React.ComponentType<Record<string, unknown>>>;

// ═══════════════════════════════════════════
// BlockAssembly — mirrors module.json#blockAssembly
// ═══════════════════════════════════════════

export interface BlockAssemblyBlock {
  id: string;
  label: string;
  group: string;
  order: number;
  icon?: string;
}

export interface BlockAssemblyGroup {
  id: string;
  label: string;
  icon: string;
}

export interface BlockAssemblyNavigation {
  groups: BlockAssemblyGroup[];
  defaultBlock: string;
  collapseBehavior?: string;
}

export interface BlockServiceBinding {
  services: string[];
  dtoTypes?: string[];
}

export interface BlockAssemblyConfig {
  /** "single-block" | "multi-block" — string 以兼容 JSON import 的 type widening */
  mode: string;
  shell: string;
  navigation: BlockAssemblyNavigation;
  blocks: BlockAssemblyBlock[];
  serviceBindings?: Record<string, BlockServiceBinding>;
  stateContract?: { shared: string[]; isolated: string[] };
}

/** i18n key map: block ID / group ID → translation key */
export interface BlockNavKeyMap {
  labels: Record<string, string>;
  groups: Record<string, string>;
}

/**
 * Derive MainNavItem[] from BlockAssembly config + i18n key map.
 * Used by createModuleLayout when blockAssembly is provided.
 */
export function deriveNavItems(
  assembly: BlockAssemblyConfig,
  keyMap: BlockNavKeyMap,
  t: (key: string) => string,
): { id: string; label: string; section: string; href: string; icon: string }[] {
  const { navigation, blocks } = assembly;
  return blocks
    .slice()
    .sort((a, b) => {
      const groupIdxA = navigation.groups.findIndex((g) => g.id === a.group);
      const groupIdxB = navigation.groups.findIndex((g) => g.id === b.group);
      if (groupIdxA !== groupIdxB) return groupIdxA - groupIdxB;
      return a.order - b.order;
    })
    .map((s) => {
      const group = navigation.groups.find((g) => g.id === s.group);
      return {
        id: s.id,
        label: t(keyMap.labels[s.id] || s.id),
        section: t(keyMap.groups[s.group] || s.group),
        href: `/${s.id}`,
        icon: s.icon || group?.icon || 'FileText',
      };
    });
}

/**
 * Generate React Router `<Route>` elements from block components and nav items.
 *
 * Blocks are registered into `window.aliothBlockComponents` as a side effect
 * (matching the prototype contract), so workspace dock consumers and audit
 * scripts see a populated registry.
 *
 * @param blockComponents — map of block ID to React component
 * @param navItems — module nav items (determines route order and path)
 * @param metas — optional per-block metadata (name, icon)
 * @returns Route element array — spread into a parent `<Route>` or `<Routes>`
 */
export function createBlockRoutes(
  blockComponents: BlockComponentMap,
  navItems: MainNavItem[],
  metas?: Record<string, BlockRouteMeta>,
): React.ReactElement[] {
  const blockIds = navItems.filter((item) => blockComponents[item.id]).map((item) => item.id);

  // Bridge the global registry contract
  for (const [id, Comp] of Object.entries(blockComponents)) {
    const meta = metas?.[id];
    registerBlock(id, Comp, { name: meta?.name, icon: meta?.icon });
  }
  registerBlockOrder(blockIds);

  const routes: React.ReactElement[] = [];
  const firstId = blockIds[0];
  const firstItem = navItems.find((i) => i.id === firstId);

  // Index redirect to first block
  if (firstId && firstItem) {
    const firstPath = firstItem.href?.replace(/^\//, '') || firstId;
    routes.push(<Route key="@@index" index element={<Navigate to={firstPath} replace />} />);
  }

  // Block routes in nav order
  for (const item of navItems) {
    const id = item.id;
    const Comp = blockComponents[id];
    if (!Comp) continue;
    const path = item.href?.replace(/^\//, '') || id;
    routes.push(<Route key={id} path={path} element={<Comp />} />);
  }

  // Wildcard fallback — unknown paths redirect to first block
  if (firstId) {
    const firstPath = navItems.find((i) => i.id === firstId)?.href?.replace(/^\//, '') || firstId;
    routes.push(<Route key="@@wildcard" path="*" element={<Navigate to={firstPath} replace />} />);
  }

  return routes;
}
