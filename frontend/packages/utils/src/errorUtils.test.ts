import { describe, it, expect } from 'vitest';
import { extractFieldErrorMessages, getErrorMessage } from './errorUtils';

describe('errorUtils', () => {
  describe('extractFieldErrorMessages', () => {
    it('extracts simple error messages', () => {
      const errors = {
        name: { message: '字段名称不能为空' },
        code: { message: '编码不能为空' },
      };
      const messages = extractFieldErrorMessages(errors);
      expect(messages).toEqual(['字段名称不能为空', '编码不能为空']);
    });

    it('extracts nested error messages', () => {
      const errors = {
        referenceConfig: {
          localKey: { message: '本表外键字段不能为空' },
        },
      };
      const messages = extractFieldErrorMessages(errors);
      expect(messages).toEqual(['本表外键字段不能为空']);
    });

    it('skips empty messages', () => {
      const errors = {
        name: { message: '' },
        code: { message: '有效消息' },
      };
      const messages = extractFieldErrorMessages(errors);
      expect(messages).toEqual(['有效消息']);
    });

    it('returns empty array for empty errors', () => {
      expect(extractFieldErrorMessages({})).toEqual([]);
    });

    it('skips null/undefined values', () => {
      const errors = {
        name: null,
        code: { message: 'test' },
      };
      const messages = extractFieldErrorMessages(errors);
      expect(messages).toEqual(['test']);
    });
  });

  describe('getErrorMessage', () => {
    it('extracts message from Error instance', () => {
      expect(getErrorMessage(new Error('something broke'))).toBe('something broke');
    });

    it('returns string as-is', () => {
      expect(getErrorMessage('direct error')).toBe('direct error');
    });

    it('extracts message from object with message field', () => {
      expect(getErrorMessage({ message: 'object error' })).toBe('object error');
    });

    it('returns fallback for unknown types', () => {
      expect(getErrorMessage(42)).toBe('components.error.unknown');
    });
  });
});
