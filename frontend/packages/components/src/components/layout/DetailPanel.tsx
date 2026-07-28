//! DetailPanel · 详情面板
//!
//! 匹配 design-system.css 的 .detail-panel-header / .detail-panel-body / .detail-panel-footer

import * as React from "react";
import { X, Trash2 } from "lucide-react";
import { cn } from "../../lib/utils";
import { StatusBadge, type StatusBadgeProps } from "../ui/status-badge";
import { Button } from "../ui/button";

export interface DetailMetaItem {
  label: string;
  value: React.ReactNode;
}

export interface DetailPanelProps {
  /** 面板标题 */
  title: string;
  /** 是否展开（默认 true）；false 时不渲染任何内容 */
  open?: boolean;
  /** 状态徽章 */
  status?: {
    label: string;
    variant: StatusBadgeProps["variant"];
  };
  /** 副标题/ID */
  subtitle?: React.ReactNode;
  /** 元信息网格数据 */
  metaItems?: DetailMetaItem[];
  /** 自定义主体内容 */
  children?: React.ReactNode;
  /** 底部操作区 */
  footer?: React.ReactNode;
  /** 关闭回调 */
  onClose?: () => void;
  /** 编辑回调 */
  onEdit?: () => void;
  /** 删除回调 */
  onDelete?: () => void;
  className?: string;
}

export function DetailPanel({
  title,
  open = true,
  status,
  subtitle,
  metaItems,
  children,
  footer,
  onClose,
  onEdit,
  onDelete,
  className,
}: DetailPanelProps) {
  if (!open) return null;
  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-4 border-b border-border bg-muted flex-shrink-0">
        <div className="flex items-center gap-3 min-w-0">
          {status && (
            <StatusBadge variant={status.variant} label={status.label} />
          )}
          <span className="text-sm font-semibold text-foreground truncate">
            {title}
          </span>
          {onDelete && (
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10 flex-shrink-0"
              onClick={(e) => {
                e.stopPropagation();
                onDelete();
              }}
              title="删除"
            >
              <Trash2 className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>
        {onClose && (
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 flex-shrink-0"
            onClick={onClose}
          >
            <X className="w-4 h-4" />
          </Button>
        )}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
        {/* 主标题区 */}
        <div>
          <h2
            className="text-xl font-bold text-foreground tracking-tight"
            style={{ fontFamily: "var(--font-display, inherit)" }}
          >
            {title}
          </h2>
          {subtitle && (
            <p className="text-xs text-muted-foreground font-mono mt-1">
              {subtitle}
            </p>
          )}
        </div>

        {/* 元信息网格 */}
        {metaItems && metaItems.length > 0 && (
          <div className="grid grid-cols-2 gap-x-4 gap-y-3">
            {metaItems.map((item, idx) => (
              <div key={idx} className="min-w-0">
                <div className="text-xs text-muted-foreground uppercase tracking-wider font-medium">
                  {item.label}
                </div>
                <div className="text-sm text-foreground mt-0.5 truncate">
                  {item.value}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 自定义内容 */}
        {children}
      </div>

      {/* Footer */}
      {(footer || onEdit) && (
        <div className="flex items-center gap-3 px-5 py-4 border-t border-border bg-muted flex-shrink-0">
          {onEdit && (
            <Button size="sm" onClick={onEdit}>
              编辑
            </Button>
          )}
          <div className="flex-1" />
          {footer}
        </div>
      )}
    </div>
  );
}
