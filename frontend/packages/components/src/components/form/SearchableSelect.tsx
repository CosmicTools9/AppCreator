//! 可搜索下拉选框
//!
//! 使用原生 div 绝对定位 + overflow-y-auto，支持键盘导航（↑↓ Enter Esc）。
//! 智能展开方向：底部空间不足时自动向上展开。

import { useState, useMemo, useRef, useEffect, useCallback } from "react";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Check, ChevronDown } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";

export interface SearchableSelectOption {
  value: string;
  label: string;
}

export interface SearchableSelectProps {
  options: SearchableSelectOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  className?: string;
  disabled?: boolean;
  /** 已废弃，保留以兼容现有调用 */
  noPortal?: boolean;
  /** 搜索文本变化回调（用于外部防抖/分页） */
  onSearchChange?: (search: string) => void;
  /** 滚动到底部回调（用于加载更多） */
  onScrollEnd?: () => void;
  /** 加载更多中（显示底部 loading） */
  loadingMore?: boolean;
}

const MAX_DISPLAY_ITEMS = 100;

export function SearchableSelect({
  options,
  value,
  onChange,
  placeholder,
  searchPlaceholder,
  emptyText,
  className,
  disabled,
  onSearchChange,
  onScrollEnd,
  loadingMore,
}: SearchableSelectProps) {
  const t = useT();
  const effectivePlaceholder = placeholder ?? t("common.pleaseSelect");
  const effectiveSearchPlaceholder = searchPlaceholder ?? t("common.search");
  const effectiveEmptyText = emptyText ?? t("common.empty");
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const [placement, setPlacement] = useState<"bottom" | "top">("bottom");
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const selectedLabel = useMemo(
    () => options.find((o) => o.value === value)?.label ?? effectivePlaceholder,
    [options, value, effectivePlaceholder],
  );

  const filteredOptions = useMemo(() => {
    if (!search.trim()) return options.slice(0, MAX_DISPLAY_ITEMS);
    const query = search.toLowerCase();
    return options
      .filter((o) => o.label.toLowerCase().includes(query))
      .slice(0, MAX_DISPLAY_ITEMS);
  }, [options, search]);

  // 搜索变化时重置高亮到第一项
  useEffect(() => {
    setHighlightedIndex(0);
  }, [search]);

  // 点击外部关闭
  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setOpen(false);
        setSearch("");
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open]);

  const handleSelect = useCallback((selectedValue: string) => {
    onChange(selectedValue);
    setOpen(false);
    setSearch("");
  }, [onChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!open) return;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setHighlightedIndex((prev) =>
          prev >= filteredOptions.length - 1 ? 0 : prev + 1
        );
        break;
      case "ArrowUp":
        e.preventDefault();
        setHighlightedIndex((prev) =>
          prev <= 0 ? filteredOptions.length - 1 : prev - 1
        );
        break;
      case "Enter":
        e.preventDefault();
        if (filteredOptions.length > 0 && highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
          handleSelect(filteredOptions[highlightedIndex].value);
        }
        break;
      case "Escape":
        e.preventDefault();
        setOpen(false);
        setSearch("");
        break;
    }
  }, [open, filteredOptions, highlightedIndex, handleSelect]);

  // 高亮项滚动到可视区域
  useEffect(() => {
    if (!open || !listRef.current) return;
    const item = itemRefs.current[highlightedIndex];
    if (item) {
      item.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [highlightedIndex, open]);

  return (
    <div ref={containerRef} className="relative" onKeyDown={handleKeyDown}>
      <Button
        type="button"
        variant="outline"
        role="combobox"
        aria-expanded={open}
        disabled={disabled}
        className={cn(
          "w-full justify-between font-normal",
          !value && "text-muted-foreground",
          className,
        )}
        onClick={() => {
          if (!open && containerRef.current) {
            const rect = containerRef.current.getBoundingClientRect();
            const spaceBelow = window.innerHeight - rect.bottom;
            // 下拉框预估高度：最大 320px (max-h-80) + 搜索框 ~48px
            const estimatedHeight = 368;
            if (spaceBelow < estimatedHeight) {
              setPlacement("top");
            } else {
              setPlacement("bottom");
            }
          }
          setOpen((prev) => !prev);
        }}
      >
        <span className="truncate">{selectedLabel}</span>
        <ChevronDown className={cn("gl-2 h-4 w-4 shrink-0 opacity-50 transition-transform", open && "rotate-180")} />
      </Button>

      {open && (
        <div className={cn(
          "absolute z-[100] w-full rounded-md border bg-popover shadow-lg flex flex-col max-h-80",
          placement === "top" ? "bottom-full mb-1" : "top-full mt-1"
        )}>
          <div className="flex items-center border-b px-3 py-2 bg-muted/50 shrink-0">
            <Input
              placeholder={effectiveSearchPlaceholder}
              value={search}
              onChange={(e) => {
                const v = e.target.value;
                setSearch(v);
                onSearchChange?.(v);
              }}
              className="h-8 border-0 bg-background focus-visible:ring-0 focus-visible:ring-offset-0 px-2 rounded-sm"
              autoFocus
            />
          </div>
          <div ref={listRef} className="overflow-y-auto overscroll-contain p-1" onScroll={(e) => {
              const el = e.currentTarget;
              if (el.scrollHeight - el.scrollTop - el.clientHeight < 40) {
                onScrollEnd?.();
              }
            }}>
            {filteredOptions.length === 0 ? (
              <div className="py-4 text-center text-sm text-muted-foreground">
                {effectiveEmptyText}
              </div>
            ) : (
              <>  {filteredOptions.map((option, index) => (
                <button
                  key={option.value}
                  ref={(el) => { itemRefs.current[index] = el; }}
                  type="button"
                  onClick={() => handleSelect(option.value)}
                  onMouseEnter={() => setHighlightedIndex(index)}
                  className={cn(
                    "relative flex w-full cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors",
                    value === option.value
                      ? "bg-accent text-accent-foreground"
                      : index === highlightedIndex
                        ? "bg-muted text-foreground"
                        : "bg-transparent hover:bg-muted hover:text-foreground",
                  )}
                >
                  <Check
                    className={cn(
                      "mr-2 h-4 w-4",
                      value === option.value ? "opacity-100" : "opacity-0",
                    )}
                  />
                  {option.label}
                </button>
              ))}
              {loadingMore && (
                <div className="py-2 text-center text-xs text-muted-foreground">
                  加载更多...
                </div>
              )}
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
