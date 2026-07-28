/**
 * AliothStudio Design Tokens
 *
 * W3C Design Tokens Community Group format
 * https://design-tokens.github.io/community-group/format/
 */

// Token categories
export * from './colors';
export * from './typography';
export * from './spacing';
export * from './shadows';
export * from './animation';
export * from './breakpoints';

// Token type definitions
export interface DesignToken {
  $value: string | number | string[] | number[];
  $type: 'color' | 'dimension' | 'fontFamily' | 'fontWeight' | 'duration' | 'cubicBezier' | 'number';
  $description?: string;
}

export interface TokenGroup {
  [key: string | number]: DesignToken | TokenGroup;
}
