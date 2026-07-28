import type { TokenGroup } from './index';

/**
 * Spacing Tokens
 *
 * Based on 4px (0.25rem) base unit
 */
export const spacing: TokenGroup = {
  space: {
    0: { $value: '0px', $type: 'dimension' },
    0.5: { $value: '0.125rem', $type: 'dimension', $description: '2px' },
    1: { $value: '0.25rem', $type: 'dimension', $description: '4px' },
    1.5: { $value: '0.375rem', $type: 'dimension', $description: '6px' },
    2: { $value: '0.5rem', $type: 'dimension', $description: '8px' },
    2.5: { $value: '0.625rem', $type: 'dimension', $description: '10px' },
    3: { $value: '0.75rem', $type: 'dimension', $description: '12px' },
    3.5: { $value: '0.875rem', $type: 'dimension', $description: '14px' },
    4: { $value: '1rem', $type: 'dimension', $description: '16px' },
    5: { $value: '1.25rem', $type: 'dimension', $description: '20px' },
    6: { $value: '1.5rem', $type: 'dimension', $description: '24px' },
    7: { $value: '1.75rem', $type: 'dimension', $description: '28px' },
    8: { $value: '2rem', $type: 'dimension', $description: '32px' },
    9: { $value: '2.25rem', $type: 'dimension', $description: '36px' },
    10: { $value: '2.5rem', $type: 'dimension', $description: '40px' },
    11: { $value: '2.75rem', $type: 'dimension', $description: '44px' },
    12: { $value: '3rem', $type: 'dimension', $description: '48px' },
    14: { $value: '3.5rem', $type: 'dimension', $description: '56px' },
    16: { $value: '4rem', $type: 'dimension', $description: '64px' },
    20: { $value: '5rem', $type: 'dimension', $description: '80px' },
    24: { $value: '6rem', $type: 'dimension', $description: '96px' },
    28: { $value: '7rem', $type: 'dimension', $description: '112px' },
    32: { $value: '8rem', $type: 'dimension', $description: '128px' },
    36: { $value: '9rem', $type: 'dimension', $description: '144px' },
    40: { $value: '10rem', $type: 'dimension', $description: '160px' },
    44: { $value: '11rem', $type: 'dimension', $description: '176px' },
    48: { $value: '12rem', $type: 'dimension', $description: '192px' },
    52: { $value: '13rem', $type: 'dimension', $description: '208px' },
    56: { $value: '14rem', $type: 'dimension', $description: '224px' },
    60: { $value: '15rem', $type: 'dimension', $description: '240px' },
    64: { $value: '16rem', $type: 'dimension', $description: '256px' },
    72: { $value: '18rem', $type: 'dimension', $description: '288px' },
    80: { $value: '20rem', $type: 'dimension', $description: '320px' },
    96: { $value: '24rem', $type: 'dimension', $description: '384px' },
  },
};
