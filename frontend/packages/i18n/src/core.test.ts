import { describe, it, expect } from 'vitest';
import { I18nCore, createProperNounSet } from './core';

describe('I18nCore', () => {
  it('uses default locale from config', () => {
    const i18n = new I18nCore();
    expect(i18n.locale).toBe('zh-CN');
  });

  it('uses custom default locale', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    expect(i18n.locale).toBe('en');
  });

  it('returns fallback locale', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    expect(i18n.fallbackLocale).toBe('en');
  });

  it('returns key as fallback when not found', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    expect(i18n.t('missing.key')).toBe('missing.key');
  });

  it('returns custom fallback when provided', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    expect(i18n.t('missing.key', undefined, { fallback: '??' })).toBe('??');
  });

  it('loads dictionary and translates', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    i18n.loadDictionary('en', { hello: 'Hello', nested: { greeting: 'Hi there' } });
    expect(i18n.t('hello')).toBe('Hello');
    expect(i18n.t('nested.greeting')).toBe('Hi there');
  });

  it('interpolates parameters', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    i18n.loadDictionary('en', { greeting: 'Hello, {name}!' });
    expect(i18n.t('greeting', { name: 'World' })).toBe('Hello, World!');
  });

  it('falls back to default locale dictionary', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    i18n.loadDictionary('en', { common: 'Common' });
    i18n.locale = 'fr';
    expect(i18n.t('common')).toBe('Common');
  });

  it('formats dates', () => {
    const i18n = new I18nCore({ defaultLocale: 'zh-CN' });
    const date = new Date(2024, 0, 15);
    const result = i18n.formatDate(date, { year: 'numeric', month: 'long', day: 'numeric' });
    expect(result).toContain('2024');
    expect(result).toContain('1');
    expect(result).toContain('15');
  });

  it('formats numbers', () => {
    const i18n = new I18nCore({ defaultLocale: 'zh-CN' });
    const result = i18n.formatNumber(1234.5, { maximumFractionDigits: 0 });
    expect(result).toContain('1,235');
  });

  it('formats currency', () => {
    const i18n = new I18nCore({ defaultLocale: 'zh-CN' });
    const result = i18n.formatCurrency(100, 'CNY');
    expect(result).toContain('100');
    expect(result).toContain('¥');
  });

  it('formats relative time', () => {
    const i18n = new I18nCore({ defaultLocale: 'zh-CN' });
    // Just verify it returns a string without throwing
    const result = i18n.formatRelativeTime(-5, 'minute');
    expect(typeof result).toBe('string');
  });

  it('loads dictionary with default config', () => {
    const dict = { greeting: '你好' };
    const i18n = new I18nCore({ defaultLocale: 'zh-CN', defaultDictionary: dict });
    expect(i18n.t('greeting')).toBe('你好');
  });

  it('unloads dictionary', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    i18n.loadDictionary('en', { key: 'value' });
    expect(i18n.t('key')).toBe('value');
    i18n.unloadDictionary('en');
    expect(i18n.t('key')).toBe('key');
  });

  it('merges dictionaries on load', () => {
    const i18n = new I18nCore({ defaultLocale: 'en' });
    i18n.loadDictionary('en', { a: '1' });
    i18n.loadDictionary('en', { b: '2' });
    expect(i18n.t('a')).toBe('1');
    expect(i18n.t('b')).toBe('2');
  });
});

describe('createProperNounSet', () => {
  it('creates empty set', () => {
    const set = createProperNounSet();
    expect(set.knownKeys.size).toBe(0);
  });

  it('creates set with initial keys', () => {
    const set = createProperNounSet(['Alioth', 'ERP']);
    expect(set.knownKeys.has('Alioth')).toBe(true);
    expect(set.knownKeys.has('ERP')).toBe(true);
  });

  it('registers new keys', () => {
    const set = createProperNounSet();
    set.register('CRM', 'SCM');
    expect(set.knownKeys.has('CRM')).toBe(true);
    expect(set.knownKeys.has('SCM')).toBe(true);
  });
});
