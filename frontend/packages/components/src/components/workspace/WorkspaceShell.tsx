/**
 * WorkspaceShell · 统一右侧工作区外壳
 *
 * 封装 Sheet 组件，统一所有右侧工作区的视觉体验：
 * - 宽度：移动端 75%，桌面端 25%
 * - 头部：标题区 + 关闭按钮（Sheet 自带）
 * - 内容区：自动计算剩余高度，支持内部滚动
 */

import * as React from "react";
import { cn } from "../../lib/utils";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "../ui/sheet";

export interface WorkspaceShellProps {
  /** 是否打开 */
  isOpen: boolean;
  /** 关闭回调 */
  onClose: () => void;
  /** 标题（可包含图标和徽章） */
  title: React.ReactNode;
  /** 面板内容 */
  children: React.ReactNode;
  /** 自定义类名 */
  className?: string;
}

export const WorkspaceShell = React.forwardRef<
  HTMLDivElement,
  WorkspaceShellProps
>(({ isOpen, onClose, title, children, className }, ref) => {
  return (
    <Sheet open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <SheetContent
        ref={ref}
        side="right"
        className={cn("p-0 w-3/4 sm:w-1/4 max-w-none", className)}
      >
        <SheetHeader className="border-b px-6 py-4">
          <SheetTitle className="flex items-center gap-2 text-base">
            {title}
          </SheetTitle>
        </SheetHeader>
        <div className="overflow-hidden" style={{ height: 'calc(100vh - 65px)' }}>
          {children}
        </div>
      </SheetContent>
    </Sheet>
  );
});

WorkspaceShell.displayName = "WorkspaceShell";
