//! Footer · 应用底栏组件
//!
//! 轻量级全局页脚，提供版权信息、导航链接和版本号展示。

import * as React from "react";
import { cn } from "../../lib/utils";

export interface FooterLink {
  label: string;
  href: string;
}

export interface FooterProps {
  /** 版权信息，为空时不显示版权文本 */
  copyright?: string;
  /** 中间链接列表 */
  links?: FooterLink[];
  /** 右侧版本号 */
  version?: string;
  className?: string;
}

export function Footer({
  copyright,
  links,
  version,
  className,
}: FooterProps): React.ReactElement {
  return (
    <footer
      className={cn(
        "hidden md:flex shrink-0 h-10 items-center justify-between border-t bg-card px-4 md:px-6",
        "text-xs text-muted-foreground",
        className,
      )}
    >
      {/* 左侧：版权 */}
      <span className="truncate">{copyright}</span>

      {/* 中间：链接 */}
      {links && links.length > 0 && (
        <nav className="hidden md:flex items-center gap-4">
          {links.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="hover:text-foreground transition-colors"
              target="_blank"
              rel="noopener noreferrer"
            >
              {link.label}
            </a>
          ))}
        </nav>
      )}

      {/* 右侧：版本 */}
      {version && (
        <span className="hidden sm:inline truncate">{version}</span>
      )}
    </footer>
  );
}

Footer.displayName = "Footer";
