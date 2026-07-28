import React, { ReactElement } from 'react';
import { render as rtlRender, RenderOptions } from '@testing-library/react';
import { I18nProvider } from '@alioth/i18n';

interface AllTheProvidersProps {
  children: React.ReactNode;
}

/**
 * 测试环境全局 Provider
 *
 * 包裹所有测试所需的上下文提供者，确保组件在测试中的行为与生产环境一致。
 * 当前包含：
 * - I18nProvider（mock 实现，提供 useI18nContext 上下文）
 *
 * TODO: 添加 ThemeProvider 以完整模拟运行时环境
 * （需先解决 next-themes 在 jsdom 中的 hydration 问题）
 */
function AllTheProviders({ children }: AllTheProvidersProps) {
  return <I18nProvider>{children}</I18nProvider>;
}

function render(ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return rtlRender(ui, { wrapper: AllTheProviders, ...options });
}

// Re-export everything from RTL
export * from '@testing-library/react';

// Override render export
export { render };

// Export provider for custom setups
export { AllTheProviders };
