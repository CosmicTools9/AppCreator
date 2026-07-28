/**
 * useDateFnsLocale · 根据 i18n locale 获取对应的 date-fns locale
 *
 * 避免在 schedule 组件中硬编码 zhCN，使日期格式随语言切换自动适配。
 */

import { useLocale } from "@alioth/i18n";
import { zhCN, enUS } from "date-fns/locale";
import type { Locale } from "date-fns";

export function useDateFnsLocale(): Locale {
  const { locale } = useLocale();
  return locale === "zh-CN" ? zhCN : enUS;
}
