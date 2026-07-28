import * as React from "react";
import { PackageOpen } from "lucide-react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";

/**
 * 空状态组件属性
 */
interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

/**
 * 空状态组件
 *
 * 在数据为空时显示友好的提示和可选的操作引导。
 *
 * @example
 * ```tsx
 * <EmptyState
 *   title="暂无数据"
 *   description="点击按钮创建第一条数据"
 *   action={{ label: "创建", onClick: handleCreate }}
 * />
 * ```
 */
const EmptyState = React.forwardRef<HTMLDivElement, EmptyStateProps>(
  ({ icon, title, description, action, className }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "flex min-h-52 flex-col justify-center py-12 px-4",
          className,
        )}
      >
        <div className="mb-4 text-muted-foreground flex justify-center">
          {icon ?? <PackageOpen className="h-12 w-12" aria-hidden="true" />}
        </div>
        <h3 className="mb-2 text-lg font-semibold text-foreground text-center">{title}</h3>
        {description && (
          <p className="mb-6 max-w-sm text-sm text-muted-foreground text-center mx-auto">
            {description}
          </p>
        )}
        {action && (
          <div className="flex justify-center">
            <Button variant="outline" onClick={action.onClick}>
              {action.label}
            </Button>
          </div>
        )}
      </div>
    );
  },
);

EmptyState.displayName = "EmptyState";

export { EmptyState };
export type { EmptyStateProps };
