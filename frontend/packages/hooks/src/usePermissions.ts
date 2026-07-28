/**
 * @description NGAC Permission Hook
 *
 * Provides a simple hook for frontend components to check whether the
 * current user has a specific permission on a resource type.
 *
 * Permissions are fetched from `/api/auth/permissions` after login and
 * cached in a Jotai atom (TTL: 5 minutes).
 *
 * @example
 * ```tsx
 * const canCreate = usePermission("engineers", "create");
 * if (!canCreate) return <AccessDenied />;
 * ```
 */

import { useAtom, atom } from "jotai";
import { useEffect } from "react";

/** Permission matrix shape: { resourceType: { action: boolean } } */
export type PermissionMap = Record<string, Record<string, boolean>>;

/** Jotai atom storing the cached permission map */
const permissionsAtom = atom<PermissionMap>({});

/** Timestamp of last fetch */
const lastFetchAtom = atom<number>(0);

/** Cache TTL in milliseconds (5 minutes) */
const CACHE_TTL = 5 * 60 * 1000;

/**
 * Fetches the full permission map from the backend.
 */
async function fetchPermissions(): Promise<PermissionMap> {
  try {
    const base = import.meta.env.VITE_GATEWAY_API_URL || "";
    const response = await fetch(`${base}/api/auth/permissions`, {
      credentials: "include",
    });
    if (!response.ok) return {};
    return await response.json();
  } catch {
    console.warn("[usePermissions] Failed to fetch permissions, returning empty");
    return {};
  }
}

/**
 * Hook that provides permission checks for the current user.
 */
export function usePermission(
  resourceType: string,
  action: string,
): boolean {
  const [permissions, setPermissions] = useAtom(permissionsAtom);
  const [lastFetch, setLastFetch] = useAtom(lastFetchAtom);

  useEffect(() => {
    const now = Date.now();
    if (now - lastFetch > CACHE_TTL) {
      fetchPermissions().then((map) => {
        setPermissions(map);
        setLastFetch(now);
      });
    }
  }, [lastFetch, setPermissions, setLastFetch]);

  return !!permissions[resourceType]?.[action];
}

/**
 * Hook that returns all permissions for a resource type.
 */
export function useResourcePermissions(
  resourceType: string,
): Record<string, boolean> {
  const [permissions, setPermissions] = useAtom(permissionsAtom);
  const [lastFetch, setLastFetch] = useAtom(lastFetchAtom);

  useEffect(() => {
    const now = Date.now();
    if (now - lastFetch > CACHE_TTL) {
      fetchPermissions().then((map) => {
        setPermissions(map);
        setLastFetch(now);
      });
    }
  }, [lastFetch, setPermissions, setLastFetch]);

  return permissions[resourceType] ?? {};
}

/**
 * Manually refresh the cached permission map.
 */
export const refreshPermissions = async (): Promise<PermissionMap> => {
  return await fetchPermissions();
};
