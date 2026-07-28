import { renderHook } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { shallowEqual, deepEqual } from './useFormFieldMemo';

describe('shallowEqual', () => {
  it('returns true for identical primitives', () => {
    expect(shallowEqual(1, 1)).toBe(true);
    expect(shallowEqual('a', 'a')).toBe(true);
  });

  it('returns true for same reference', () => {
    const obj = { a: 1 };
    expect(shallowEqual(obj, obj)).toBe(true);
  });

  it('returns false for different values', () => {
    expect(shallowEqual({ a: 1 }, { a: 2 })).toBe(false);
  });

  it('returns true for objects with same top-level values', () => {
    expect(shallowEqual({ a: 1, b: 2 }, { a: 1, b: 2 })).toBe(true);
  });

  it('returns false for objects with different keys', () => {
    expect(shallowEqual({ a: 1 }, { a: 1, b: 2 })).toBe(false);
  });

  it('handles null values', () => {
    expect(shallowEqual(null, null)).toBe(true);
    expect(shallowEqual({ a: 1 }, null)).toBe(false);
  });
});

describe('deepEqual', () => {
  it('returns true for identical values', () => {
    expect(deepEqual({ a: { b: 1 } }, { a: { b: 1 } })).toBe(true);
  });

  it('returns false for different nested values', () => {
    expect(deepEqual({ a: { b: 1 } }, { a: { b: 2 } })).toBe(false);
  });

  it('returns false for objects with different keys', () => {
    expect(deepEqual({ a: 1 }, { a: 1, b: 2 })).toBe(false);
  });

  it('handles null values', () => {
    expect(deepEqual(null, { a: 1 })).toBe(false);
  });
});
