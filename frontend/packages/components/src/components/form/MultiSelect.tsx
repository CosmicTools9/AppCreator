/**
 * MultiSelect · 通用多选下拉
 *
 * 与 SearchableSelect 形态一致的下拉面板，但允许选中多项。
 * 已选项以可移除的 Badge 形式展示在触发器内。
 */

import { useState, useMemo, useRef, useEffect, useCallback } from "react";
import { Check, ChevronDown, X } from "lucide-react";
import { cn } from "../../lib/utils";
import { Input } from "../ui/input";
import { Badge } from "../ui/badge";
import { useT } from "@alioth/i18n";

export interface MultiSelectOption {
  value: string;
  label: string;
  /** 不可选的项（仍展示但无法选中/取消） */
  disabled?: boolean;
}

export interface MultiSelectProps {
  options: MultiSelectOption[];
  value: string[];
  onChange: (value: string[]) => void;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  disabled?: boolean;
  className?: string;
  /** 触发器最大可见 chip 数量；超出折叠为 +N */
  maxChips?: number;
}

const MAX_DISPLAY_ITEMS = 200;

export function MultiSelect({
  options,
  value,
  onChange,
  placeholder,
  searchPlaceholder,
  emptyText,
  disabled,
  className,
  maxChips = 3,
}: MultiSelectProps) {
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

  const valueSet = useMemo(() => new Set(value), [value]);
  const selectedOptions = useMemo(
    () => options.filter((o) => valueSet.has(o.value)),
    [options, valueSet],
  );

  const filteredOptions = useMemo(() => {
    if (!search.trim()) return options.slice(0, MAX_DISPLAY_ITEMS);
    const q = search.toLowerCase();
    return options
      .filter((o) => o.label.toLowerCase().includes(q))
      .slice(0, MAX_DISPLAY_ITEMS);
  }, [options, search]);

  useEffect(() => {
    setHighlightedIndex(0);
  }, [search]);

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

  const toggleValue = useCallback(
    (selectedValue: string, optionDisabled?: boolean) => {
      if (optionDisabled) return;
      if (valueSet.has(selectedValue)) {
        onChange(value.filter((v) => v !== selectedValue));
      } else {
        onChange([...value, selectedValue]);
      }
    },
    [onChange, value, valueSet],
  );

  const removeValue = useCallback(
    (e: React.MouseEvent, removeKey: string) => {
      e.stopPropagation();
      if (disabled) return;
      onChange(value.filter((v) => v !== removeKey));
    },
    [disabled, onChange, value],
  );

  const clearAll = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (disabled) return;
      onChange([]);
    },
    [disabled, onChange],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!open) return;
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev >= filteredOptions.length - 1 ? 0 : prev + 1,
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev <= 0 ? filteredOptions.length - 1 : prev - 1,
          );
          break;
        case "Enter":
          e.preventDefault();
          if (
            filteredOptions.length > 0 &&
            highlightedIndex >= 0 &&
            highlightedIndex < filteredOptions.length
          ) {
            const opt = filteredOptions[highlightedIndex];
            toggleValue(opt.value, opt.disabled);
          }
          break;
        case "Escape":
          e.preventDefault();
          setOpen(false);
          setSearch("");
          break;
      }
    },
    [open, filteredOptions, highlightedIndex, toggleValue],
  );

  useEffect(() => {
    if (!open || !listRef.current) return;
    const item = itemRefs.current[highlightedIndex];
    if (item) item.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [highlightedIndex, open]);

  const visibleChips = selectedOptions.slice(0, maxChips);
  const hiddenCount = selectedOptions.length - visibleChips.length;

  return (
    <div ref={containerRef} className="relative" onKeyDown={handleKeyDown}>
      <div
        role="combobox"
        aria-expanded={open}
        aria-disabled={disabled}
        tabIndex={disabled ? -1 : 0}
        className={cn(
          "flex min-h-9 w-full items-center justify-between gap-1 rounded-md border border-input bg-transparent px-3 py-1.5 text-sm shadow-sm",
          "hover:border-foreground/40 focus-within:border-ring focus-within:ring-1 focus-within:ring-ring/40",
          disabled && "cursor-not-allowed opacity-50 bg-muted",
          className,
        )}
        onClick={() => {
          if (disabled) return;
          if (!open && containerRef.current) {
            const rect = containerRef.current.getBoundingClientRect();
            const spaceBelow = window.innerHeight - rect.bottom;
            const estimatedHeight = 368;
            setPlacement(spaceBelow < estimatedHeight ? "top" : "bottom");
          }
          setOpen((prev) => !prev);
        }}
      >
        <div className="flex flex-1 flex-wrap items-center gap-1 overflow-hidden">
          {selectedOptions.length === 0 ? (
            <span className="text-muted-foreground truncate">
              {effectivePlaceholder}
            </span>
          ) : (
            <>
              {visibleChips.map((opt) => (
                <Badge
                  key={opt.value}
                  variant="secondary"
                  className="gap-1 px-1.5 py-0 text-[11px] font-normal"
                >
                  <span className="truncate max-w-[120px]">{opt.label}</span>
                  {!disabled && (
                    <button
                      type="button"
                      onClick={(e) => removeValue(e, opt.value)}
                      className="ml-0.5 rounded-sm hover:bg-foreground/10 p-0.5"
                      aria-label="remove"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  )}
                </Badge>
              ))}
              {hiddenCount > 0 && (
                <Badge
                  variant="outline"
                  className="px-1.5 py-0 text-[11px] font-normal"
                >
                  +{hiddenCount}
                </Badge>
              )}
            </>
          )}
        </div>

        <div className="flex items-center gap-1 shrink-0">
          {value.length > 0 && !disabled && (
            <button
              type="button"
              onClick={clearAll}
              className="rounded-sm p-0.5 hover:bg-muted text-muted-foreground"
              aria-label="clear all"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
          <ChevronDown
            className={cn(
              "h-4 w-4 opacity-50 transition-transform",
              open && "rotate-180",
            )}
          />
        </div>
      </div>

      {open && (
        <div
          className={cn(
            "absolute z-[100] w-full rounded-md border bg-popover shadow-lg flex flex-col max-h-80",
            placement === "top" ? "bottom-full mb-1" : "top-full mt-1",
          )}
        >
          <div className="flex items-center border-b px-3 py-2 bg-muted/50 shrink-0">
            <Input
              placeholder={effectiveSearchPlaceholder}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-8 border-0 bg-background focus-visible:ring-0 focus-visible:ring-offset-0 px-2 rounded-sm"
              autoFocus
            />
          </div>
          <div
            ref={listRef}
            className="overflow-y-auto overscroll-contain p-1"
          >
            {filteredOptions.length === 0 ? (
              <div className="py-4 text-center text-sm text-muted-foreground">
                {effectiveEmptyText}
              </div>
            ) : (
              filteredOptions.map((option, index) => {
                const isSelected = valueSet.has(option.value);
                return (
                  <button
                    key={option.value}
                    ref={(el) => {
                      itemRefs.current[index] = el;
                    }}
                    type="button"
                    disabled={option.disabled}
                    onClick={() => toggleValue(option.value, option.disabled)}
                    onMouseEnter={() => setHighlightedIndex(index)}
                    className={cn(
                      "relative flex w-full cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors",
                      option.disabled && "opacity-50 cursor-not-allowed",
                      !option.disabled &&
                        (isSelected
                          ? "bg-accent text-accent-foreground"
                          : index === highlightedIndex
                            ? "bg-muted text-foreground"
                            : "bg-transparent hover:bg-muted hover:text-foreground"),
                    )}
                  >
                    <div
                      className={cn(
                        "mr-2 flex h-4 w-4 shrink-0 items-center justify-center rounded border",
                        isSelected
                          ? "border-primary bg-primary text-primary-foreground"
                          : "border-input bg-background",
                      )}
                    >
                      {isSelected && <Check className="h-3 w-3" />}
                    </div>
                    <span className="truncate">{option.label}</span>
                  </button>
                );
              })
            )}
          </div>
        </div>
      )}
    </div>
  );
}

MultiSelect.displayName = "MultiSelect";