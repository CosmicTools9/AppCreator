import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useThrottle, useThrottleCallback, useDebounceCallback } from './useThrottle';

describe('useThrottle', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns initial value immediately', () => {
    const { result } = renderHook(() => useThrottle('initial', 1000));
    expect(result.current).toBe('initial');
  });

  it('updates value after interval passes', () => {
    const { result, rerender } = renderHook(
      ({ value }) => useThrottle(value, 1000),
      { initialProps: { value: 'initial' } },
    );

    // Change value
    rerender({ value: 'updated' });

    // Should still be initial (within throttle interval)
    expect(result.current).toBe('initial');

    // Advance time past interval
    act(() => { vi.advanceTimersByTime(1000); });

    // Should now be updated
    expect(result.current).toBe('updated');
  });

  it('throttles rapid updates', () => {
    const { result, rerender } = renderHook(
      ({ value }) => useThrottle(value, 500),
      { initialProps: { value: 'a' } },
    );

    expect(result.current).toBe('a');

    rerender({ value: 'b' });
    rerender({ value: 'c' });
    rerender({ value: 'd' });

    // Still throttled
    expect(result.current).toBe('a');

    // Advance past interval
    act(() => { vi.advanceTimersByTime(500); });

    // Should get last value
    expect(result.current).toBe('d');
  });

  it('updates immediately when enough time has passed', () => {
    const { result, rerender } = renderHook(
      ({ value }) => useThrottle(value, 500),
      { initialProps: { value: 'first' } },
    );

    // Wait for initial throttle effect to settle
    act(() => { vi.advanceTimersByTime(500); });

    // Now lastUpdated.current = 500
    // Advance more and update value
    act(() => { vi.advanceTimersByTime(500); });

    rerender({ value: 'second' });

    // Should now have the updated value via immediate path
    // (since timeElapsed = 1000 - 500 = 500 >= 500)
    act(() => { vi.advanceTimersByTime(0); }); // flush microtasks
    expect(result.current).toBe('second');
  });
});

describe('useThrottleCallback', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('calls callback immediately on first invocation', () => {
    const fn = vi.fn();
    const { result } = renderHook(() => useThrottleCallback(fn, 1000));

    act(() => { result.current('arg1'); });
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith('arg1');
  });

  it('throttles subsequent calls within interval', () => {
    const fn = vi.fn();
    const { result } = renderHook(() => useThrottleCallback(fn, 500));

    act(() => { result.current('first'); });
    act(() => { result.current('second'); });
    act(() => { result.current('third'); });

    // Only first call should have executed
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith('first');
  });

  it('executes last pending call after interval', () => {
    const fn = vi.fn();
    const { result } = renderHook(() => useThrottleCallback(fn, 500));

    act(() => { result.current('first'); });
    act(() => { result.current('second'); });

    expect(fn).toHaveBeenCalledTimes(1);

    act(() => { vi.advanceTimersByTime(500); });

    // The pending 'second' should now be executed
    expect(fn).toHaveBeenCalledTimes(2);
    expect(fn).toHaveBeenLastCalledWith('second');
  });
});

describe('useDebounceCallback', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('executes callback after delay', () => {
    const fn = vi.fn();
    const { result } = renderHook(() => useDebounceCallback(fn, 300));

    act(() => { result.current('hello'); });
    expect(fn).not.toHaveBeenCalled();

    act(() => { vi.advanceTimersByTime(300); });
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith('hello');
  });

  it('resets timer on rapid calls', () => {
    const fn = vi.fn();
    const { result } = renderHook(() => useDebounceCallback(fn, 300));

    act(() => { result.current('first'); });
    act(() => { vi.advanceTimersByTime(200); });

    // Call again before the 300ms delay
    act(() => { result.current('second'); });
    act(() => { vi.advanceTimersByTime(200); });

    // Should not have fired yet (reset by second call)
    expect(fn).not.toHaveBeenCalled();

    act(() => { vi.advanceTimersByTime(100); });

    // Should have fired with the last value
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith('second');
  });
});
