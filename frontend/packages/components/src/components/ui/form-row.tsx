//! FormRow · 双列表单行布局
//!
//! 匹配 design-system.css 的 .form-row

import * as React from "react";
import { cn } from "../../lib/utils";

export interface FormRowProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
  columns?: 1 | 2 | 3;
}

export function FormRow({
  className,
  children,
  columns = 2,
  ...props
}: FormRowProps) {
  const gridCols = {
    1: "grid-cols-1",
    2: "grid-cols-1 md:grid-cols-2",
    3: "grid-cols-1 md:grid-cols-3",
  };

  return (
    <div
      className={cn("grid gap-4", gridCols[columns], className)}
      {...props}
    >
      {children}
    </div>
  );
}
