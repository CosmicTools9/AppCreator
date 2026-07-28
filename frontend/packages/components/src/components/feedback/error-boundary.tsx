import * as React from "react";
import { XCircle, RefreshCw, Home, ChevronDown, ChevronUp } from "lucide-react";
import { Button } from "../ui/button";
import { useT } from "@alioth/i18n";

/**
 * 错误回退组件属性
 */
export interface ErrorFallbackProps {
  error: Error;
  resetErrorBoundary: () => void;
  title?: string;
  description?: string;
  showDetails?: boolean;
  onGoHome?: () => void;
  onRefresh?: () => void;
}

/**
 * 错误回退 UI 组件
 *
 * 在错误边界捕获错误时显示的友好错误提示。
 */
export function ErrorFallback({
  error,
  resetErrorBoundary,
  title,
  description,
  showDetails = false,
  onGoHome,
  onRefresh,
}: ErrorFallbackProps): React.ReactElement | null {
  const t = useT();
  const displayTitle = title || t('components.error.title');
  const displayDescription = description || t('components.error.description');
  const [detailsOpen, setDetailsOpen] = React.useState(false);

  return (
    <div className="flex min-h-52 items-center justify-center p-6">
      <div className="max-w-lg w-full text-center">
        <div className="mb-6">
          <div className="w-20 h-20 mx-auto bg-destructive/10 rounded-full flex items-center justify-center">
            <XCircle className="w-10 h-10 text-destructive" />
          </div>
        </div>

        <h2 className="text-2xl font-bold text-foreground mb-2">
          {displayTitle}
        </h2>

        <p className="text-muted-foreground mb-6">
          {error?.message || displayDescription}
        </p>

        <div className="flex flex-wrap gap-3 justify-center mb-6">
          <Button onClick={resetErrorBoundary} variant="default" className="gap-2">
            <RefreshCw className="w-4 h-4" />
            {t('components.action.retry')}
          </Button>
          {onGoHome && (
            <Button onClick={onGoHome} variant="outline" className="gap-2">
              <Home className="w-4 h-4" />
              {t('components.error.backHome')}
            </Button>
          )}
          {onRefresh && (
            <Button onClick={onRefresh} variant="outline" className="gap-2">
              <RefreshCw className="w-4 h-4" />
              {t('components.error.refreshPage')}
            </Button>
          )}
        </div>

        {showDetails && error && (
          <div className="border border-border rounded-lg overflow-hidden text-left">
            <button
              onClick={() => setDetailsOpen((prev) => !prev)}
              className="w-full px-4 py-3 flex items-center justify-between bg-muted hover:bg-accent transition-colors"
            >
              <span className="text-sm font-medium text-foreground">
                {t('components.error.details')}
              </span>
              {detailsOpen ? (
                <ChevronUp className="w-4 h-4 text-muted-foreground" />
              ) : (
                <ChevronDown className="w-4 h-4 text-muted-foreground" />
              )}
            </button>

            {detailsOpen && (
              <div className="px-4 py-3">
                <div className="mb-3">
                  <p className="text-xs font-medium text-muted-foreground mb-1">
                    {t('components.error.errorName')}
                  </p>
                  <p className="text-sm font-mono text-destructive break-all">
                    {error.name}
                  </p>
                </div>

                <div className="mb-3">
                  <p className="text-xs font-medium text-muted-foreground mb-1">
                    {t('components.error.errorMessage')}
                  </p>
                  <pre className="text-sm font-mono text-foreground bg-muted p-2 rounded overflow-auto max-h-32">
                    {error.message}
                  </pre>
                </div>

                {error.stack && (
                  <div className="mb-3">
                    <p className="text-xs font-medium text-muted-foreground mb-1">
                      {t('components.error.stackTrace')}
                    </p>
                    <pre className="text-xs font-mono text-muted-foreground bg-muted p-2 rounded overflow-auto max-h-48">
                      {error.stack}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * 错误边界组件属性
 */
export interface ErrorBoundaryProps {
  children: React.ReactNode;
  FallbackComponent?: React.ComponentType<ErrorFallbackProps>;
  onReset?: () => void;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  resetKeys?: unknown[];
  showDetails?: boolean;
  fallback?: React.ReactNode;
}

/**
 * 错误边界组件状态
 */
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

/**
 * 错误边界组件
 *
 * 捕获子组件中的 JavaScript 错误，防止整个应用崩溃。
 * 显示友好的错误提示，并提供重试按钮。
 */
class ErrorBoundary extends React.Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error, errorInfo: null };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    this.setState({ errorInfo });

    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    console.error("ErrorBoundary caught error:", error, errorInfo);
  }

  componentDidUpdate(prevProps: ErrorBoundaryProps): void {
    if (this.state.hasError && this.props.resetKeys && prevProps.resetKeys) {
      const hasResetKeyChanged = this.props.resetKeys.some(
        (key, index) => key !== prevProps.resetKeys?.[index],
      );

      if (hasResetKeyChanged) {
        this.resetErrorBoundary();
      }
    }
  }

  resetErrorBoundary = () => {
    this.props.onReset?.();
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  goHome = (): void => {
    if (typeof window !== "undefined") {
      window.location.href = "/";
    }
  };

  refreshPage = (): void => {
    if (typeof window !== "undefined") {
      window.location.reload();
    }
  };

  render() {
    if (this.state.hasError && this.state.error) {
      const { fallback, FallbackComponent, showDetails } = this.props;

      if (fallback !== undefined) {
        return fallback;
      }

      const Fallback = FallbackComponent || ErrorFallback;
      return (
        <Fallback
          error={this.state.error}
          resetErrorBoundary={this.resetErrorBoundary}
          showDetails={showDetails}
          onGoHome={this.goHome}
          onRefresh={this.refreshPage}
        />
      );
    }

    return this.props.children;
  }
}

/**
 * ErrorBoundary HOC
 * 为组件添加错误边界保护
 */
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<ErrorBoundaryProps, "children">,
): (props: P) => React.ReactElement | null {
  function WrappedComponent(props: P): React.ReactElement | null {
    return (
      <ErrorBoundary {...errorBoundaryProps}>
        <Component {...props} />
      </ErrorBoundary>
    );
  }

  const componentName = Component.displayName || Component.name || "Component";
  WrappedComponent.displayName = `withErrorBoundary(${componentName})`;

  return WrappedComponent;
}

export { ErrorBoundary };
