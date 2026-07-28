import * as React from "react";
import { cn } from "../../lib/utils";

export interface QuickLinkProps {
  label: string;
  description?: string;
  icon: React.ComponentType<{ className?: string }>;
  href?: string;
  onClick?: () => void;
  className?: string;
}

export function QuickLink({
  label,
  description,
  icon: Icon,
  href,
  onClick,
  className,
}: QuickLinkProps) {
  const Comp = href ? "a" : "button";

  return (
    <Comp
      href={href}
      onClick={onClick}
      className={cn(
        "flex flex-col items-center gap-2 p-4 rounded-xl border bg-card hover:shadow-md transition-all hover:-translate-y-0.5 text-center cursor-pointer",
        className,
      )}
    >
      <div className="p-2 rounded-lg bg-primary/10 text-primary">
        <Icon className="w-5 h-5" />
      </div>
      <span className="text-sm font-medium text-foreground">{label}</span>
      {description && (
        <span className="text-xs text-muted-foreground line-clamp-2">
          {description}
        </span>
      )}
    </Comp>
  );
}
