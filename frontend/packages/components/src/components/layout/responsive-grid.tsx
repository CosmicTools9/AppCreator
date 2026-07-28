import * as React from "react";
import { cn } from "../../lib/utils";

/**
 * 响应式网格列数配置
 */
interface ResponsiveGridCols {
  mobile?: number; // < 768px (default: 1)
  tablet?: number; // 768px - 1024px (default: 2)
  desktop?: number; // > 1024px (default: 3)
}

/**
 * 响应式网格组件属性
 */
interface ResponsiveGridProps {
  children: React.ReactNode;
  className?: string;
  cols?: ResponsiveGridCols;
  gap?: 2 | 3 | 4 | 6 | 8;
}

/**
 * 列数对应的 CSS 类名
 */
const colClasses: Record<number, string> = {
  1: "grid-cols-1",
  2: "grid-cols-1 md:grid-cols-2",
  3: "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
  4: "grid-cols-1 sm:grid-cols-2 lg:grid-cols-4",
  5: "grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5",
  6: "grid-cols-2 sm:grid-cols-3 lg:grid-cols-6",
};

/**
 * 响应式网格组件
 *
 * 在移动/平板/桌面端自动适配列数的网格布局。
 *
 * @example
 * ```tsx
 * <ResponsiveGrid cols={{ mobile: 1, tablet: 2, desktop: 3 }} gap={4}>
 *   <Card>...</Card>
 *   <Card>...</Card>
 *   <Card>...</Card>
 * </ResponsiveGrid>
 * ```
 */
const ResponsiveGrid = React.forwardRef<HTMLDivElement, ResponsiveGridProps>(
  ({ children, className, cols, gap = 4 }, ref) => {
    const mobileCols = cols?.mobile ?? 1;
    const tabletCols = cols?.tablet ?? 2;
    const desktopCols = cols?.desktop ?? 3;

    // 构建响应式类名
    const getGridColsClass = () => {
      // 如果三端列数相同，简化类名
      if (mobileCols === tabletCols && tabletCols === desktopCols) {
        return colClasses[mobileCols]?.split(" ")[0] || "grid-cols-1";
      }

      // 构建完整的响应式类名
      const classes: string[] = [];

      // 移动端（基础）
      classes.push(colClasses[mobileCols]?.split(" ")[0] || "grid-cols-1");

      // 平板端
      if (tabletCols !== mobileCols) {
        const tabletClass = colClasses[tabletCols]
          ?.split(" ")
          .find((c) => c.startsWith("md:"));
        if (tabletClass) classes.push(tabletClass);
      }

      // 桌面端
      if (desktopCols !== tabletCols) {
        const desktopClass = colClasses[desktopCols]
          ?.split(" ")
          .find((c) => c.startsWith("lg:"));
        if (desktopClass) classes.push(desktopClass);
      }

      return classes.join(" ");
    };

    return (
      <div
        ref={ref}
        className={cn(
          "grid",
          getGridColsClass(),
          {
            "gap-2": gap === 2,
            "gap-3": gap === 3,
            "gap-4": gap === 4,
            "gap-6": gap === 6,
            "gap-8": gap === 8,
          },
          className,
        )}
      >
        {children}
      </div>
    );
  },
);

ResponsiveGrid.displayName = "ResponsiveGrid";

export { ResponsiveGrid };
export type { ResponsiveGridProps, ResponsiveGridCols };
