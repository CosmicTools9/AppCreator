import * as React from "react";
import { Loader2 } from "lucide-react";
import { useT } from "@alioth/i18n";
import { cn } from "../../lib/utils";

/**
 * 加载遮罩组件属性
 */
interface LoadingOverlayProps {
  isLoading: boolean;
  children: React.ReactNode;
  className?: string;
  spinner?: React.ReactNode;
}

/**
 * 加载遮罩组件
 *
 * 当 isLoading 为 true 时显示半透明遮罩，阻止用户交互。
 * 支持自定义加载动画。
 *
 * @example
 * ```tsx
 * <LoadingOverlay isLoading={isSubmitting}>
 *   <form>...表单内容...</form>
 * </LoadingOverlay>
 * ```
 */
const LoadingOverlay = React.forwardRef<HTMLDivElement, LoadingOverlayProps>(
  ({ isLoading, children, className, spinner }, ref) => {
    const t = useT();
    return (
      <div ref={ref} className={cn("relative", className)}>
        {children}

        {isLoading && (
          <div
            className={cn(
              "absolute inset-0 z-50 flex items-center justify-center",
              "bg-background/80 backdrop-blur-sm",
            )}
            aria-busy="true"
            aria-live="polite"
          >
            {spinner ?? (
              <Loader2
                className="h-8 w-8 animate-spin text-primary"
                aria-hidden="true"
              />
            )}
            <span className="sr-only">{t("components.loading")}</span>
          </div>
        )}
      </div>
    );
  },
);

LoadingOverlay.displayName = "LoadingOverlay";

export { LoadingOverlay };
export type { LoadingOverlayProps };
