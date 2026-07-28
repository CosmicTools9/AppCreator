//! BizBadges · 彩色徽章组件
//!
//! 跨模块统一的颜色映射与 React 渲染函数。
//! 所有模块 import 此工具而非自建色表，确保一致性。

import type { ReactNode } from "react";

export interface BadgeConfig {
  label: string;
  cls: string;
}

/** 运输方式色表（order_cate） */
export const ORDER_CATE_COLORS: Record<string, BadgeConfig> = {
  sea: { label: "海运", cls: "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300" },
  ocean: { label: "海运", cls: "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300" },
  air: { label: "空运", cls: "bg-indigo-100 text-indigo-700 border-indigo-200 dark:bg-indigo-900/30 dark:text-indigo-300" },
  land: { label: "陆运", cls: "bg-amber-100 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-300" },
  road: { label: "公路", cls: "bg-amber-100 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-300" },
  rail: { label: "铁路", cls: "bg-purple-100 text-purple-700 border-purple-200 dark:bg-purple-900/30 dark:text-purple-300" },
};

const FALLBACK: BadgeConfig = { label: "", cls: "bg-slate-100 text-slate-600 border-slate-200" };

function lookup(map: Record<string, BadgeConfig>, value: string | null): BadgeConfig {
  return map[value ?? ""] ?? { label: value ?? "", cls: FALLBACK.cls };
}

/** 渲染运输类型徽章 */
export function OrderCateBadge({ value }: { value: string | null }): ReactNode {
  if (!value) return "—";
  const cfg = lookup(ORDER_CATE_COLORS, value);
  return <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border ${cfg.cls}`}>{cfg.label}</span>;
}
