import * as React from 'react';
import { ThemeProvider as NextThemesProvider } from 'next-themes';

/**
 * 主题提供器组件属性
 */
export type ThemeProviderProps = React.ComponentProps<typeof NextThemesProvider>;

/**
 * 主题提供器组件
 *
 * 包装 next-themes 的 ThemeProvider，提供系统主题检测和自动切换功能。
 * 支持 light、dark、system 三种主题模式。
 *
 * @example
 * ```tsx
 * <ThemeProvider>
 *   <App />
 * </ThemeProvider>
 * ```
 */
export function ThemeProvider({ children, ...props }: ThemeProviderProps): React.ReactElement | null {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="light"
      enableSystem={true}
      disableTransitionOnChange={false}
      {...props}
    >
      {children}
    </NextThemesProvider>
  );
}

