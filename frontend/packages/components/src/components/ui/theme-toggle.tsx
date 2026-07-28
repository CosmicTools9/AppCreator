import * as React from 'react';
import { useTheme } from 'next-themes';
import { Button } from './button';
import { Sun, Moon } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useT } from '@alioth/i18n';

/**
 * 主题切换按钮组件
 *
 * 提供太阳/月亮图标动画的主题切换按钮。
 * 点击按钮在浅色和深色主题之间切换。
 *
 * @example
 * ```tsx
 * <ThemeToggle />
 * ```
 */
export function ThemeToggle({
  className,
  ...props
}: React.ComponentProps<typeof Button>): React.ReactElement | null {
  const { theme, setTheme } = useTheme();
  const t = useT();

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
      className={cn("relative h-9 w-9", className)}
      {...props}
    >
      <Sun
        className="h-4 w-4 rotate-0 scale-100 transition-transform dark:-rotate-90 dark:scale-0"
        aria-hidden="true"
      />
      <Moon
        className="absolute h-4 w-4 rotate-90 scale-0 transition-transform dark:rotate-0 dark:scale-100"
        aria-hidden="true"
      />
      <span className="sr-only">{t("components.theme.toggle")}</span>
    </Button>
  );
}

ThemeToggle.displayName = "ThemeToggle";
