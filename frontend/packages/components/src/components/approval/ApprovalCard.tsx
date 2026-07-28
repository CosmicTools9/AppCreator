/**
 * ApprovalCard · 审批单据卡片（纵向布局版）
 *
 * 针对 docked 窄面板（~320–400px）重新设计：
 * - 纵向三行布局，每行信息不挤压
 * - 标题独占一行，不再截断
 * - 底部操作栏：金额（左）+ 状态/操作（右）
 */

import * as React from 'react';
import { Check, X, XCircle } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Button } from '../ui/button';
import { useT } from '@alioth/i18n';
import { Badge } from '../ui/badge';
import type { ApprovalCardProps, ApprovalStatus } from './types';

function useStatusConfig(): Record<
  ApprovalStatus,
  { label: string; badgeClass: string; dotClass: string }
> {
  const t = useT();
  return React.useMemo(
    () => ({
      pending: {
        label: t('components.approval.status.pending'),
        badgeClass:
          'bg-warning/10 dark:bg-warning/20 text-warning/80 border-warning/20 hover:bg-warning/20',
        dotClass: 'bg-warning',
      },
      approved: {
        label: t('components.approval.status.approved'),
        badgeClass:
          'bg-success/10 dark:bg-success/20 text-success/80 border-success/20 hover:bg-success/20',
        dotClass: 'bg-success',
      },
      rejected: {
        label: t('components.approval.status.rejected'),
        badgeClass:
          'bg-destructive/10 dark:bg-destructive/20 text-destructive/80 border-destructive/20 hover:bg-destructive/20',
        dotClass: 'bg-destructive',
      },
    }),
    [t],
  );
}

export const ApprovalCard = React.forwardRef<HTMLDivElement, ApprovalCardProps>(
  ({ item, onApprove, onReject, onClick, className }, ref) => {
    const t = useT();
    const statusConfig = useStatusConfig();
    const config = statusConfig[item.status];
    const isPending = item.status === 'pending';

    const avatarText =
      item.avatar ?? (item.applicant ? item.applicant.charAt(0).toUpperCase() : '?');

    return (
      <div
        ref={ref}
        className={cn(
          'rounded-xl border bg-card p-4 hover:border-border/80 transition-colors',
          className,
        )}
      >
        {/* 第 1 行：头像 + 标题 */}
        <div className="flex items-start gap-3">
          <div className="w-9 h-9 rounded-lg bg-muted flex items-center justify-center text-xs font-bold text-muted-foreground shrink-0 mt-0.5">
            {avatarText}
          </div>
          <div className="flex-1 min-w-0">
            <button
              onClick={() => onClick?.(item)}
              className="font-semibold text-foreground text-left hover:text-primary transition-colors text-sm leading-snug"
            >
              {item.title}
            </button>
          </div>
        </div>

        {/* 第 2 行：元信息 */}
        <div className="mt-2 pl-12 text-xs text-muted-foreground leading-relaxed">
          <span>{item.applicant}</span>
          {item.dept && (
            <>
              <span className="mx-1.5 text-border">·</span>
              <span>{item.dept}</span>
            </>
          )}
          <span className="mx-1.5 text-border">·</span>
          <span>{item.time}</span>
          {item.type && (
            <>
              <span className="mx-1.5 text-border">·</span>
              <span className="text-muted-foreground/70">{item.type}</span>
            </>
          )}
        </div>

        {/* 第 3 行：底部栏 — 编号 + 状态 + 操作 */}
        <div className="mt-3 pl-12 flex items-center justify-between gap-3">
          {/* 左侧：审批编号 */}
          <div className="min-w-0">
            {item.code ? (
              <span className="text-sm font-semibold text-foreground font-display">
                {item.code}
              </span>
            ) : (
              <span className="text-xs text-muted-foreground/60">—</span>
            )}
          </div>

          {/* 右侧：状态 + 操作 */}
          <div className="flex items-center gap-2 shrink-0">
            <Badge
              variant="outline"
              className={cn(
                'text-xs px-2 py-0.5 rounded-full border font-medium',
                config.badgeClass,
              )}
            >
              <span className={cn('w-1.5 h-1.5 rounded-full mr-1.5', config.dotClass)} />
              {config.label}
            </Badge>

            {isPending ? (
              <div className="flex items-center gap-1.5">
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 px-2.5 text-xs font-medium text-muted-foreground hover:bg-destructive/10 hover:text-destructive hover:border-destructive/20 transition-colors"
                  onClick={(e) => {
                    e.stopPropagation();
                    onReject?.(item.id);
                  }}
                >
                  <X className="w-3 h-3 mr-0.5" />
                  {t('components.approval.action.reject')}
                </Button>
                <Button
                  size="sm"
                  className="h-7 px-2.5 text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
                  onClick={(e) => {
                    e.stopPropagation();
                    onApprove?.(item.id);
                  }}
                >
                  <Check className="w-3 h-3 mr-0.5" />
                  {t('components.approval.action.approve')}
                </Button>
              </div>
            ) : (
              <span className="text-xs text-muted-foreground inline-flex items-center gap-1">
                {item.status === 'approved' ? (
                  <span className="inline-flex items-center gap-1 text-success">
                    <Check className="w-3 h-3" />
                    {t('components.approval.action.processed')}
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 text-destructive/70">
                    <XCircle className="w-3 h-3" />
                    {t('components.approval.status.rejected')}
                  </span>
                )}
              </span>
            )}
          </div>
        </div>
      </div>
    );
  },
);

ApprovalCard.displayName = 'ApprovalCard';
