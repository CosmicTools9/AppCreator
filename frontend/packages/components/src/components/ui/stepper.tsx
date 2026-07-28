import * as React from 'react';
import { Check } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useT } from "@alioth/i18n";

/**
 * 步骤数据类型
 */
export interface Step {
  id: string;
  label: string;
  description?: string;
}

/**
 * 步骤指示器组件属性
 */
export interface StepperProps {
  steps: Step[];
  currentStep: number; // 0-based index
  className?: string;
}

/**
 * 步骤指示器组件
 *
 * 显示多步骤流程的进度，支持完成/当前/待办三种状态。
 * 移动端横向滚动，桌面端完整展示。
 *
 * @example
 * ```tsx
 * const steps = [
 *   { id: "1", label: "第一步", description: "填写基本信息" },
 *   { id: "2", label: "第二步", description: "确认订单" },
 *   { id: "3", label: "第三步", description: "完成支付" },
 * ];
 *
 * <Stepper steps={steps} currentStep={1} />
 * ```
 */
export function Stepper({ steps, currentStep, className }: StepperProps): React.ReactElement | null {
  const t = useT();
  return (
    <nav aria-label={t("components.stepper.progress")} className={cn("w-full", className)}>
      <ol
        className="flex items-start gap-2 overflow-x-auto pb-2 md:gap-4"
        role="list"
      >
        {steps.map((step, index) => {
          const isCompleted = index < currentStep;
          const isCurrent = index === currentStep;
          const isPending = index > currentStep;

          return (
            <React.Fragment key={step.id}>
              <li
                className="flex shrink-0 items-center"
                aria-current={isCurrent ? "step" : undefined}
              >
                {/* 步骤指示器 */}
                <div
                  className={cn(
                    "flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-medium transition-colors",
                    {
                      "bg-primary text-primary-foreground": isCompleted,
                      "border-2 border-primary text-primary": isCurrent,
                      "border border-muted-foreground/30 text-muted-foreground":
                        isPending,
                    },
                  )}
                  aria-label={t("components.stepper.step", { index: index + 1, label: step.label })}
                >
                  {isCompleted ? (
                    <Check className="h-4 w-4" aria-hidden="true" />
                  ) : (
                    <span>{index + 1}</span>
                  )}
                </div>

                {/* 标签区域 - 桌面端显示 */}
                <div className="gl-2 hidden md:block">
                  <p
                    className={cn(
                      "text-sm font-medium",
                      isCurrent ? "text-foreground" : "text-muted-foreground",
                    )}
                  >
                    {step.label}
                  </p>
                  {step.description && (
                    <p className="text-xs text-muted-foreground">
                      {step.description}
                    </p>
                  )}
                </div>
              </li>

              {/* 连接线 */}
              {index < steps.length - 1 && (
                <li
                  className={cn(
                    "mx-2 h-0.5 w-8 shrink-0 md:mx-2 md:w-12",
                    isCompleted ? "bg-primary" : "bg-muted",
                  )}
                  aria-hidden="true"
                  role="separator"
                />
              )}
            </React.Fragment>
          );
        })}
      </ol>
    </nav>
  );
}

