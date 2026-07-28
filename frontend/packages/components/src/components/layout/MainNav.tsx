import * as React from "react";
import { cn } from "../../lib/utils";
import { ScrollArea } from "../ui/scroll-area";
import { Badge } from "../ui/badge";
import { ChevronDown, ChevronRight } from "lucide-react";
import { DynamicIcon } from "../ui/dynamic-icon";

export interface MainNavItem {
  id: string;
  label: string;
  href: string;
  /** Lucide 图标名称字符串（如 "Shield"） */
  icon?: string;
  badge?: string | number;
  children?: MainNavItem[];
  /** 分组标题，相同 section 的项会被归为一组 */
  section?: string;
  /** 默认展开子导航 */
  defaultExpanded?: boolean;
}

export interface MainNavProps {
  items: MainNavItem[];
  activeItemId?: string;
  onItemClick?: (item: MainNavItem) => void;
  collapsed?: boolean;
  className?: string;
}

const NavItem = ({
  item,
  activeItemId,
  onItemClick,
  collapsed,
}: {
  item: MainNavItem;
  activeItemId?: string;
  onItemClick?: (item: MainNavItem) => void;
  collapsed?: boolean;
}) => {
  const [expanded, setExpanded] = React.useState(item.defaultExpanded ?? false);
  const isActive = item.id === activeItemId;
  const hasChildren = item.children && item.children.length > 0;
  const handleClick = () => {
    if (hasChildren) {
      setExpanded(!expanded);
    }
    onItemClick?.(item);
  };
  return (
    <div className="space-y-1">
      <button
        type="button"
        onClick={handleClick}
        className={cn(
          "flex items-center rounded-md text-sm font-medium transition-colors",
          collapsed
            ? "justify-center h-9 w-9 mx-auto my-0.5 px-2"
            : "w-[calc(100%-1rem)] mx-2 gap-2.5 py-2 px-4",
          isActive
            ? "bg-primary/10 text-primary font-semibold"
            : "text-muted-foreground hover:bg-accent hover:text-foreground",
        )}
      >
        {item.icon && (
          <DynamicIcon
            name={item.icon}
            className="h-4 w-4 shrink-0"
          />
        )}
        {!collapsed && (
          <>
            <span className="flex-1 truncate text-left">{item.label}</span>
            {item.badge !== undefined && (
              <Badge
                variant="secondary"
                className={cn(
                  "h-5 min-w-5 px-1 text-xs border-0",
                  isActive ? "bg-primary/15 text-primary" : "bg-muted text-muted-foreground",
                )}
              >
                {item.badge}
              </Badge>
            )}
            {hasChildren && (
              <span className="gl-auto">
                {expanded ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
              </span>
            )}
          </>
        )}
      </button>
      {!collapsed && hasChildren && expanded && (
        <div className="mt-1 space-y-1 pl-4">
          {item.children!.map((child) => (
            <NavItem
              key={child.id}
              item={child}
              activeItemId={activeItemId}
              onItemClick={onItemClick}
              collapsed={collapsed}
            />
          ))}
        </div>
      )}
    </div>
  );
};

/** 按 section 分组导航项 */
function groupNavItems(items: MainNavItem[]): Array<{ section?: string; items: MainNavItem[] }> {
  const groups: Array<{ section?: string; items: MainNavItem[] }> = [];
  let currentGroup: MainNavItem[] = [];
  let currentSection: string | undefined;

  for (const item of items) {
    if (item.section !== currentSection) {
      if (currentGroup.length > 0) {
        groups.push({ section: currentSection, items: currentGroup });
      }
      currentGroup = [item];
      currentSection = item.section;
    } else {
      currentGroup.push(item);
    }
  }

  if (currentGroup.length > 0) {
    groups.push({ section: currentSection, items: currentGroup });
  }

  return groups;
}

export const MainNav = React.forwardRef<HTMLDivElement, MainNavProps>(
  ({ items, activeItemId, onItemClick, collapsed = false, className }, ref) => {
    const groups = React.useMemo(() => groupNavItems(items), [items]);

    return (
      <nav
        ref={ref}
        className={cn(
          "flex h-full flex-col border-r bg-secondary",
          collapsed ? "w-16" : "w-60",
          className,
        )}
      >
        <ScrollArea className="flex-1 px-0 py-2">
          <div className="space-y-4">
            {groups.map((group, groupIndex) => (
              <div key={group.section ?? `__ungrouped_${groupIndex}`} className="space-y-1">
                {!collapsed && group.section && (
                  <div className="px-4 py-3.5 pb-1">
                    <span className="text-[10px] font-bold uppercase tracking-[0.06em] text-muted-foreground/55">
                      {group.section}
                    </span>
                  </div>
                )}
                {group.items.map((item) => (
                  <NavItem
                    key={item.id}
                    item={item}
                    activeItemId={activeItemId}
                    onItemClick={onItemClick}
                    collapsed={collapsed}
                  />
                ))}
              </div>
            ))}
          </div>
        </ScrollArea>
      </nav>
    );
  },
);
MainNav.displayName = "MainNav";

export { NavItem };
