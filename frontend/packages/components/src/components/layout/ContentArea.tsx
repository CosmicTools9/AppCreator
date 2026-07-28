import * as React from 'react';
import { cn } from '../../lib/utils';

export interface ContentAreaProps {
  children: React.ReactNode;
  className?: string;
  padding?: 'none' | 'default' | 'large';
  /** 顶部强调条（sticky 固定在 main 顶部，滚动时停留） */
  accentBar?: React.ReactNode;
}

export const ContentArea = React.forwardRef<HTMLDivElement, ContentAreaProps>(
  ({ children, className, padding = 'default', accentBar }, ref) => {
    const paddingClasses = {
      none: '',
      default: 'px-6 pb-6',
      large: 'px-8 pb-8',
    };

    return (
      <main
        ref={ref}
        className={cn(
          'flex-1 overflow-y-auto bg-background flex flex-col relative [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:bg-muted-foreground/20 [&::-webkit-scrollbar-corner]:bg-transparent',
          paddingClasses[padding],
          className,
        )}
      >
        {accentBar && (
          <div className={cn('sticky top-0 z-10 shrink-0', padding !== 'none' && '-mx-6')}>
            {accentBar}
          </div>
        )}
        {children}
      </main>
    );
  },
);
ContentArea.displayName = 'ContentArea';
