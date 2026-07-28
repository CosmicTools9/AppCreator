import type { TokenGroup } from './index';

/**
 * Shadow Tokens (Elevation)
 */
export const shadows: TokenGroup = {
  shadow: {
    sm: {
      $value: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
      $type: 'color',
      $description: 'Small shadow for subtle elevation',
    },
    DEFAULT: {
      $value: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
      $type: 'color',
      $description: 'Default shadow',
    },
    md: {
      $value: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
      $type: 'color',
      $description: 'Medium shadow for cards',
    },
    lg: {
      $value: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
      $type: 'color',
      $description: 'Large shadow for modals',
    },
    xl: {
      $value: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
      $type: 'color',
      $description: 'Extra large shadow for dialogs',
    },
    '2xl': {
      $value: '0 25px 50px -12px rgb(0 0 0 / 0.25)',
      $type: 'color',
      $description: '2X shadow for maximum elevation',
    },
    inner: {
      $value: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)',
      $type: 'color',
      $description: 'Inner shadow for inset effects',
    },
    none: {
      $value: '0 0 #0000',
      $type: 'color',
      $description: 'No shadow',
    },
  },
  radius: {
    none: { $value: '0px', $type: 'dimension' },
    sm: { $value: '0.25rem', $type: 'dimension', $description: '4px' },
    DEFAULT: { $value: '0.25rem', $type: 'dimension', $description: '4px' },
    md: { $value: '0.5rem', $type: 'dimension', $description: '8px — match design-system.css' },
    lg: { $value: '0.75rem', $type: 'dimension', $description: '12px — match design-system.css' },
    xl: { $value: '1rem', $type: 'dimension', $description: '16px — match design-system.css' },
    '2xl': { $value: '1.25rem', $type: 'dimension', $description: '20px — match design-system.css' },
    '3xl': { $value: '1.5rem', $type: 'dimension', $description: '24px' },
    full: { $value: '9999px', $type: 'dimension', $description: 'Fully rounded' },
  },
};
