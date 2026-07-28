import { describe, it, expect } from 'vitest';
import { formatDate, formatDateTime, formatRelativeTime } from './dateUtils';

describe('dateUtils', () => {
  it('formatDate formats correctly', () => {
    const date = new Date(2024, 0, 15); // Jan 15, 2024
    const result = formatDate(date);
    expect(result).toContain('2024');
    expect(result).toContain('01');
    expect(result).toContain('15');
  });

  it('formatDate accepts string input', () => {
    const result = formatDate('2024-06-15T00:00:00Z');
    expect(result).toContain('2024');
  });

  it('formatDateTime includes time', () => {
    const date = new Date(2024, 0, 15, 14, 30, 0);
    const result = formatDateTime(date);
    expect(result).toContain('2024');
    expect(result).toContain('14');
    expect(result).toContain('30');
  });

  it('formatRelativeTime returns justNow for recent dates', () => {
    const now = new Date();
    const result = formatRelativeTime(now);
    expect(result).toContain('justNow');
  });

  it('formatRelativeTime returns minutesAgo', () => {
    const past = new Date(Date.now() - 5 * 60 * 1000);
    const result = formatRelativeTime(past);
    expect(result).toContain('minutesAgo');
  });

  it('formatRelativeTime returns hoursAgo', () => {
    const past = new Date(Date.now() - 3 * 60 * 60 * 1000);
    const result = formatRelativeTime(past);
    expect(result).toContain('hoursAgo');
  });

  it('formatRelativeTime returns daysAgo', () => {
    const past = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000);
    const result = formatRelativeTime(past);
    expect(result).toContain('daysAgo');
  });

  it('formatRelativeTime uses translation function when provided', () => {
    const past = new Date(Date.now() - 5 * 60 * 1000);
    const t = (key: string, params?: Record<string, string>) =>
      `${key}:${params?.minutes || ''}`;
    const result = formatRelativeTime(past, t);
    expect(result).toBe('components.date.minutesAgo:5');
  });
});
