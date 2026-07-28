import { describe, it, expect, beforeEach } from 'vitest';
import { storage, sessionStorage } from './storageUtils';

describe('storageUtils', () => {
  beforeEach(() => {
    localStorage.clear();
    window.sessionStorage.clear();
  });

  describe('storage (localStorage)', () => {
    it('sets and gets a value', () => {
      storage.set('key', { a: 1 });
      expect(storage.get('key')).toEqual({ a: 1 });
    });

    it('returns null for missing key', () => {
      expect(storage.get('nonexistent')).toBeNull();
    });

    it('removes a value', () => {
      storage.set('key', 'value');
      storage.remove('key');
      expect(storage.get('key')).toBeNull();
    });

    it('clears all values', () => {
      storage.set('a', 1);
      storage.set('b', 2);
      storage.clear();
      expect(storage.get('a')).toBeNull();
      expect(storage.get('b')).toBeNull();
    });

    it('handles complex objects', () => {
      const obj = { name: 'test', nested: { arr: [1, 2, 3] } };
      storage.set('obj', obj);
      expect(storage.get('obj')).toEqual(obj);
    });
  });

  describe('sessionStorage', () => {
    it('sets and gets a value', () => {
      sessionStorage.set('key', 'session-value');
      expect(sessionStorage.get('key')).toBe('session-value');
    });

    it('returns null for missing key', () => {
      expect(sessionStorage.get('missing')).toBeNull();
    });

    it('removes a value', () => {
      sessionStorage.set('temp', 'data');
      sessionStorage.remove('temp');
      expect(sessionStorage.get('temp')).toBeNull();
    });

    it('clears all values', () => {
      sessionStorage.set('a', 1);
      sessionStorage.set('b', 2);
      sessionStorage.clear();
      expect(sessionStorage.get('a')).toBeNull();
      expect(sessionStorage.get('b')).toBeNull();
    });
  });
});
