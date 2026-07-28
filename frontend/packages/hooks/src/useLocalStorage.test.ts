import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLocalStorage } from './useLocalStorage';

describe('useLocalStorage', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('returns initial value when key does not exist', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));
    expect(result.current[0]).toBe('default');
  });

  it('returns stored value when key exists', () => {
    localStorage.setItem('test-key', JSON.stringify('stored-value'));
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));
    expect(result.current[0]).toBe('stored-value');
  });

  it('sets and persists a string value', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));

    act(() => {
      result.current[1]('new-value');
    });

    expect(result.current[0]).toBe('new-value');
    expect(localStorage.getItem('test-key')).toBe('"new-value"');
  });

  it('sets and persists a number value', () => {
    const { result } = renderHook(() => useLocalStorage('count', 0));

    act(() => {
      result.current[1](42);
    });

    expect(result.current[0]).toBe(42);
    expect(localStorage.getItem('count')).toBe('42');
  });

  it('sets and persists an object value', () => {
    const { result } = renderHook(() => useLocalStorage<{ name: string }>('user', { name: 'alice' }));

    act(() => {
      result.current[1]({ name: 'bob' });
    });

    expect(result.current[0]).toEqual({ name: 'bob' });
    expect(JSON.parse(localStorage.getItem('user')!)).toEqual({ name: 'bob' });
  });

  it('handles invalid JSON in localStorage gracefully', () => {
    localStorage.setItem('corrupt', '{invalid json');
    const { result } = renderHook(() => useLocalStorage('corrupt', 'fallback'));
    expect(result.current[0]).toBe('fallback');
  });

  it('uses different keys independently', () => {
    const { result: r1 } = renderHook(() => useLocalStorage('key-a', 'a'));
    const { result: r2 } = renderHook(() => useLocalStorage('key-b', 'b'));

    act(() => { r1.current[1]('A'); });
    act(() => { r2.current[1]('B'); });

    expect(r1.current[0]).toBe('A');
    expect(r2.current[0]).toBe('B');
  });
});
