import type { TokenGroup } from './index';

/**
 * Animation Tokens
 */
export const animation: TokenGroup = {
  duration: {
    0: { $value: '0ms', $type: 'duration' },
    75: { $value: '75ms', $type: 'duration' },
    100: { $value: '100ms', $type: 'duration' },
    150: { $value: '150ms', $type: 'duration' },
    200: { $value: '200ms', $type: 'duration' },
    300: { $value: '300ms', $type: 'duration' },
    500: { $value: '500ms', $type: 'duration' },
    700: { $value: '700ms', $type: 'duration' },
    1000: { $value: '1000ms', $type: 'duration' },
  },
  ease: {
    linear: { $value: [0, 0, 1, 1], $type: 'cubicBezier' },
    in: { $value: [0.4, 0, 1, 1], $type: 'cubicBezier', $description: 'Ease in' },
    out: { $value: [0, 0, 0.2, 1], $type: 'cubicBezier', $description: 'Ease out' },
    'in-out': { $value: [0.4, 0, 0.2, 1], $type: 'cubicBezier', $description: 'Ease in-out' },
  },
};
