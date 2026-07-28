import * as React from "react";
import { cn } from "../../lib/utils";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "../ui/breadcrumb";
import { ScrollTabs } from "./ScrollTabs";

export interface BreadcrumbItemType {
  label: string;
  href?: string;
}

export interface TopBarProps {
  logo?: React.ReactNode;
  breadcrumbs?: BreadcrumbItemType[];
  /** 左侧模块标签页（顶栏左区，logo 之后、面包屑之前） */
  tabs?: React.ReactNode;
  actions?: React.ReactNode;
  userMenu?: React.ReactNode;
  searchSlot?: React.ReactNode;
  className?: string;
  /**
   * 变体：
   * - `"default"`（默认）: h-14, bg-background — 与模块级 TopBar 同高，避免 Gateway ↔ 模块切换跳变
   * - `"module"`: h-14, bg-background — 用于模块级 TopBar（ModuleLayout）
   */
  variant?: "default" | "module";
}

export const TopBar = React.forwardRef<HTMLDivElement, TopBarProps>(
  ({ logo, breadcrumbs, tabs, actions, userMenu, searchSlot, className }, ref) => {
    const variantClasses = "h-14 bg-background";
    return (
      <header
        ref={ref}
        className={cn(
          "flex items-center justify-between border-b px-6",
          variantClasses,
          className,
        )}
      >
        {/* 左侧: 品牌标识 + 模块标签页 + 面包屑 */}
        <div className="flex items-center gap-2 min-w-0">
          {logo && <div className="shrink-0">{logo}</div>}
          {tabs && (
            <ScrollTabs aria-label="模块标签">
              {tabs}
            </ScrollTabs>
          )}
          {breadcrumbs && breadcrumbs.length > 0 && (
            <Breadcrumb className="hidden sm:flex">
              <BreadcrumbList>
                {breadcrumbs.map((item, index) => (
                  <React.Fragment key={index}>
                    {index > 0 && <BreadcrumbSeparator />}
                    <BreadcrumbItem>
                      {index === breadcrumbs.length - 1 || !item.href ? (
                        <BreadcrumbPage>{item.label}</BreadcrumbPage>
                      ) : (
                        <BreadcrumbLink href={item.href}>
                          {item.label}
                        </BreadcrumbLink>
                      )}
                    </BreadcrumbItem>
                  </React.Fragment>
                ))}
              </BreadcrumbList>
            </Breadcrumb>
          )}
        </div>

        {/* 右侧: 搜索框 + 操作按钮 + 用户菜单 — 统一在同一 flex 容器内 */}
        <div className="flex items-center gap-3">
          {searchSlot && <div className="shrink-0">{searchSlot}</div>}
          {actions}
          {userMenu}
        </div>
      </header>
    );
  },
);
TopBar.displayName = "TopBar";
