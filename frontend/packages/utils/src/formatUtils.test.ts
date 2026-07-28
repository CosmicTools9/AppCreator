import { describe, it, expect } from 'vitest';
import { formatCurrency, formatNumber, formatFileSize, truncateText } from './formatUtils';

describe('formatUtils', () => {
  describe('formatCurrency', () => {
    it('formats CNY by default', () => {
      const result = formatCurrency(1234.56);
      expect(result).toContain('1,234.56');
    });

    it('formats with custom currency', () => {
      const result = formatCurrency(100, 'USD');
      expect(result).toContain('100');
      expect(result).toContain('US');
    });
  });

  describe('formatNumber', () => {
    it('formats number with commas', () => {
      expect(formatNumber(1234567)).toContain('1,234,567');
    });

    it('handles small numbers', () => {
      expect(formatNumber(42)).toContain('42');
    });
  });

  describe('formatFileSize', () => {
    it('formats 0 bytes', () => {
      expect(formatFileSize(0)).toBe('0 B');
    });

    it('formats KB', () => {
      expect(formatFileSize(1024)).toContain('KB');
    });

    it('formats MB', () => {
      const result = formatFileSize(1024 * 1024);
      expect(result).toContain('MB');
    });

    it('formats GB', () => {
      const result = formatFileSize(1024 * 1024 * 1024);
      expect(result).toContain('GB');
    });
  });

  describe('truncateText', () => {
    it('returns text as-is when shorter than maxLength', () => {
      expect(truncateText('hello', 10)).toBe('hello');
    });

    it('truncates and adds ellipsis', () => {
      expect(truncateText('hello world', 5)).toBe('hello...');
    });

    it('returns exact text when length equals maxLength', () => {
      expect(truncateText('hello', 5)).toBe('hello');
    });
  });
});
