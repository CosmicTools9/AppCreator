/**
 * @description Navigation menu item for sidebar/header menus
 *
 * Represents a single menu item with optional nested children.
 * Used for building dynamic navigation menus based on user permissions.
 *
 * @example
 * ```typescript
 * const menuItems: MenuItem[] = [
 *   {
 *     id: "dashboard",
 *     title: "Dashboard",
 *     path: "/dashboard",
 *     icon: "LayoutDashboard"
 *   },
 *   {
 *     id: "users",
 *     title: "User Management",
 *     path: "/users",
 *     icon: "Users",
 *     permissions: ["admin"],
 *     children: [
 *       { id: "users-list", title: "All Users", path: "/users/list" },
 *       { id: "users-roles", title: "Roles", path: "/users/roles" }
 *     ]
 *   }
 * ];
 * ```
 */
export interface MenuItem {
  /** Unique identifier for the menu item */
  id: string;
  /** Display title for the menu item */
  title: string;
  /** Navigation path/route */
  path: string;
  /** Optional icon name (Lucide icon component name) */
  icon?: string;
  /** Nested child menu items for dropdowns */
  children?: MenuItem[];
  /** Required permissions to view this menu item */
  permissions?: string[];
  /** Whether to hide this menu item from navigation */
  hidden?: boolean;
}

/**
 * @description Route configuration for dynamic routing
 *
 * Defines route metadata including component, title, and access control.
 * Used for generating route tables and permission-based route filtering.
 *
 * @example
 * ```typescript
 * const routes: RouteConfig[] = [
 *   {
 *     path: "/dashboard",
 *     component: "DashboardPage",
 *     title: "Dashboard",
 *     icon: "LayoutDashboard"
 *   },
 *   {
 *     path: "/settings",
 *     component: "SettingsPage",
 *     title: "Settings",
 *     permissions: ["admin"],
 *     children: [
 *       { path: "/settings/general", component: "GeneralSettings", title: "General" },
 *       { path: "/settings/security", component: "SecuritySettings", title: "Security" }
 *     ]
 *   }
 * ];
 * ```
 */
export interface RouteConfig {
  /** Route path pattern */
  path: string;
  /** Component name to render for this route */
  component: string;
  /** Page title for breadcrumbs and document.title */
  title: string;
  /** Optional icon for navigation */
  icon?: string;
  /** Required permissions to access this route */
  permissions?: string[];
  /** Nested child routes */
  children?: RouteConfig[];
}
