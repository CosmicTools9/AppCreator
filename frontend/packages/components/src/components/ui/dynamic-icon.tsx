import * as React from "react";
import * as LucideIcons from "lucide-react";

export interface DynamicIconProps {
  /** Lucide 图标名称字符串 */
  name: string;
  className?: string;
  /** 当 name 无法解析时的 fallback 图标名称，默认 "Package" */
  fallback?: string;
}

/**
 * DynamicIcon · 按名称动态渲染 Lucide 图标
 *
 * 统一模块 Icon 的渲染入口，以 module.json 中的 icon 字符串为唯一真相源。
 * 用法：
 *   <DynamicIcon name="Shield" className="w-5 h-5 text-primary" />
 */
export function DynamicIcon({
  name,
  className,
  fallback = "Package",
}: DynamicIconProps): React.ReactElement | null {
  const icons = LucideIcons as unknown as Record<string, React.ElementType | undefined>;
  const Icon = icons[name] ?? icons[fallback] ?? LucideIcons.Package;
  if (!Icon) return <div className={className} />;
  return <Icon className={className} />;
}
