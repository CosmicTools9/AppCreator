import type { TokenGroup } from './index';

/**
 * Typography Tokens
 */
export const typography: TokenGroup = {
  fontFamily: {
    sans: {
      $value: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'Noto Sans', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', 'WenQuanYi Micro Hei', 'sans-serif'],
      $type: 'fontFamily',
      $description: 'Primary sans-serif font stack — Inter (locally self-hosted) for Latin Display/Body; CJK handled by system fallback chain.',
    },
    mono: {
      $value: ['JetBrains Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'Liberation Mono', 'Courier New', 'monospace'],
      $type: 'fontFamily',
      $description: 'Monospace font — JetBrains Mono (locally self-hosted) for code, IDs, timestamps.',
    },
  },
  fontSize: {
    xs: { $value: '0.75rem', $type: 'dimension', $description: '12px' },
    sm: { $value: '0.875rem', $type: 'dimension', $description: '14px' },
    base: { $value: '1rem', $type: 'dimension', $description: '16px' },
    lg: { $value: '1.125rem', $type: 'dimension', $description: '18px' },
    xl: { $value: '1.25rem', $type: 'dimension', $description: '20px' },
    '2xl': { $value: '1.5rem', $type: 'dimension', $description: '24px' },
    '3xl': { $value: '1.875rem', $type: 'dimension', $description: '30px' },
    '4xl': { $value: '2.25rem', $type: 'dimension', $description: '36px' },
    '5xl': { $value: '3rem', $type: 'dimension', $description: '48px' },
  },
  fontWeight: {
    thin: { $value: 100, $type: 'fontWeight' },
    extralight: { $value: 200, $type: 'fontWeight' },
    light: { $value: 300, $type: 'fontWeight' },
    normal: { $value: 400, $type: 'fontWeight' },
    medium: { $value: 500, $type: 'fontWeight' },
    semibold: { $value: 600, $type: 'fontWeight' },
    bold: { $value: 700, $type: 'fontWeight' },
    extrabold: { $value: 800, $type: 'fontWeight' },
    black: { $value: 900, $type: 'fontWeight' },
  },
  lineHeight: {
    none: { $value: 1, $type: 'number' },
    tight: { $value: 1.25, $type: 'number' },
    snug: { $value: 1.375, $type: 'number' },
    normal: { $value: 1.5, $type: 'number' },
    relaxed: { $value: 1.625, $type: 'number' },
    loose: { $value: 2, $type: 'number' },
  },
  letterSpacing: {
    tighter: { $value: '-0.05em', $type: 'dimension' },
    tight: { $value: '-0.025em', $type: 'dimension' },
    normal: { $value: '0em', $type: 'dimension' },
    wide: { $value: '0.025em', $type: 'dimension' },
    wider: { $value: '0.05em', $type: 'dimension' },
    widest: { $value: '0.1em', $type: 'dimension' },
  },
};
