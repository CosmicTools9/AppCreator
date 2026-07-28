//! 可搜索下拉选择器
//!
//! 当选项 > 8 项时自动启用搜索输入框，< 8 项时降级为普通 Select。

import * as React from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { cn } from "../../lib/utils";
import { Button } from "./button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "./popover";
import { useT } from "@alioth/i18n";

export interface SearchSelectOption {
  value: string;
  label: string;
}

export interface SearchSelectProps {
  value: string;
  onValueChange: (value: string) => void;
  options: SearchSelectOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  disabled?: boolean;
  /** 超过此数量自动启用搜索，默认 8 */
  searchThreshold?: number;
}

export function SearchSelect({
  value,
  onValueChange,
  options,
  placeholder = "请选择…",
  searchPlaceholder,
  emptyText,
  disabled = false,
  searchThreshold = 8,
}: SearchSelectProps) {
  const t = useT();
  const [open, setOpen] = React.useState(false);
  const [searchQuery, setSearchQuery] = React.useState("");

  const selected = options.find((o) => o.value === value);
  const needsSearch = options.length > searchThreshold;

  // 根据搜索词过滤
  const filtered = needsSearch && searchQuery
    ? options.filter((o) =>
        o.label.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : options;

  const resolvedPlaceholder = placeholder;
  const resolvedSearchPlaceholder = searchPlaceholder ?? t("common.search", {}, { fallback: "搜索…" });
  const resolvedEmptyText = emptyText ?? t("common.noResults", {}, { fallback: "无匹配结果" });

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          disabled={disabled}
          className="w-full justify-between font-normal"
        >
          {selected ? selected.label : resolvedPlaceholder}
          <ChevronsUpDown className="gl-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="p-0" align="start">
        <Command shouldFilter={!needsSearch}>
          {needsSearch && (
            <CommandInput
              placeholder={resolvedSearchPlaceholder}
              value={searchQuery}
              onValueChange={setSearchQuery}
            />
          )}
          <CommandList>
            <CommandEmpty>{resolvedEmptyText}</CommandEmpty>
            <CommandGroup>
              {filtered.map((opt) => (
                <CommandItem
                  key={opt.value}
                  value={opt.value}
                  onSelect={(currentValue) => {
                    onValueChange(currentValue === value ? "" : currentValue);
                    setOpen(false);
                    setSearchQuery("");
                  }}
                >
                  <Check
                    className={cn(
                      "mr-2 h-4 w-4",
                      value === opt.value ? "opacity-100" : "opacity-0"
                    )}
                  />
                  {opt.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
