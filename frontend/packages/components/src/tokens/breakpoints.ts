import type { TokenGroup } from './index';

/**
 * Breakpoint Tokens
 */
export const breakpoints: TokenGroup = {
  screens: {
    sm: { $value: '640px', $type: 'dimension', $description: 'Small devices' },
    md: { $value: '768px', $type: 'dimension', $description: 'Medium devices (tablet)' },
    lg: { $value: '1024px', $type: 'dimension', $description: 'Large devices (laptop)' },
    xl: { $value: '1280px', $type: 'dimension', $description: 'Extra large (desktop)' },
    '2xl': { $value: '1536px', $type: 'dimension', $description: '2X large screens' },
  },
  zIndex: {
    0: { $value: 0, $type: 'number' },
    10: { $value: 10, $type: 'number' },
    20: { $value: 20, $type: 'number' },
    30: { $value: 30, $type: 'number' },
    40: { $value: 40, $type: 'number' },
    50: { $value: 50, $type: 'number' },
    auto: { $value: 'auto', $type: 'number' },
    dropdown: { $value: 1000, $type: 'number' },
    sticky: { $value: 1020, $type: 'number' },
    fixed: { $value: 1030, $type: 'number' },
    modalBackdrop: { $value: 1040, $type: 'number' },
    modal: { $value: 1050, $type: 'number' },
    popover: { $value: 1060, $type: 'number' },
    tooltip: { $value: 1070, $type: 'number' },
  },
};
