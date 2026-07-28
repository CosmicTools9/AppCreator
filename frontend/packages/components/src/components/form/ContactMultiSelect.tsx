/**
 * ContactMultiSelect · 联系人多选组件
 *
 * 用于站内信等场景中选择多个联系人（收件人、抄送人）。
 * 显示为可删除的 Badge 标签 + 搜索下拉面板。
 * 支持键盘导航（↑↓ Enter Esc）。
 */

import { useState, useMemo, useRef, useEffect, useCallback } from "react";
import { X, User, Check } from "lucide-react";
import { cn } from "../../lib/utils";
import { Badge } from "../ui/badge";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "../ui/command";
import { useT } from "@alioth/i18n";

export interface ContactOption {
  id: string;
  name: string;
  avatar?: string;
  type?: string;
  code?: string;
}

export interface ContactMultiSelectProps {
  value: string[];
  onChange: (value: string[]) => void;
  options: ContactOption[];
  placeholder?: string;
  label?: string;
  disabled?: boolean;
  loading?: boolean;
  emptyText?: string;
  searchPlaceholder?: string;
  className?: string;
  maxHeight?: number;
}

export function ContactMultiSelect({
  value,
  onChange,
  options,
  placeholder,
  label,
  disabled,
  loading,
  emptyText,
  searchPlaceholder,
  className,
  maxHeight = 280,
}: ContactMultiSelectProps) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedContacts = useMemo(
    () => options.filter((o) => value.includes(o.id)),
    [options, value]
  );

  const filteredOptions = useMemo(() => {
    if (!search.trim()) return options;
    const q = search.toLowerCase();
    return options.filter(
      (o) =>
        o.name.toLowerCase().includes(q) ||
        (o.code && o.code.toLowerCase().includes(q)) ||
        (o.type && o.type.toLowerCase().includes(q))
    );
  }, [options, search]);

  const toggleContact = useCallback(
    (contactId: string) => {
      if (value.includes(contactId)) {
        onChange(value.filter((id) => id !== contactId));
      } else {
        onChange([...value, contactId]);
      }
    },
    [value, onChange]
  );

  const removeContact = useCallback(
    (contactId: string) => {
      onChange(value.filter((id) => id !== contactId));
    },
    [value, onChange]
  );

  // 点击外部不关闭（由 Popover 管理），但清除搜索
  useEffect(() => {
    if (!open) setSearch("");
  }, [open]);

  return (
    <div ref={containerRef} className={cn("w-full", className)}>
      {label && (
        <label className="block text-sm font-medium text-slate-700 mb-1.5">
          {label}
        </label>
      )}

      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <div
            className={cn(
              "min-h-[38px] w-full rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm cursor-text transition-colors",
              "hover:border-slate-300 focus-within:border-blue-500 focus-within:ring-2 focus-within:ring-blue-100",
              disabled && "opacity-50 cursor-not-allowed bg-slate-50",
              className
            )}
            onClick={() => !disabled && setOpen(true)}
          >
            {selectedContacts.length === 0 ? (
              <span className="text-slate-400">
                {placeholder ?? t("contacts.multiSelect.placeholder", {}, { fallback: "请选择联系人..." })}
              </span>
            ) : (
              <div className="flex flex-wrap gap-1.5">
                {selectedContacts.map((contact) => (
                  <Badge
                    key={contact.id}
                    variant="secondary"
                    className="gap-1 px-2 py-0.5 text-xs font-normal bg-slate-100 text-slate-700 hover:bg-slate-200 transition-colors"
                  >
                    <User className="w-3 h-3" />
                    <span className="truncate max-w-[120px]">{contact.name}</span>
                    {!disabled && (
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeContact(contact.id);
                        }}
                        className="gl-0.5 rounded-sm hover:bg-slate-300/50 p-0.5"
                      >
                        <X className="w-3 h-3" />
                      </button>
                    )}
                  </Badge>
                ))}
              </div>
            )}
          </div>
        </PopoverTrigger>

        <PopoverContent
          className="w-[320px] p-0"
          align="start"
          side="bottom"
          sideOffset={4}
        >
          <Command className="overflow-hidden">
            <CommandInput
              placeholder={
                searchPlaceholder ??
                t("contacts.multiSelect.search", {}, { fallback: "搜索联系人..." })
              }
              value={search}
              onValueChange={setSearch}
              className="border-0 border-b border-slate-100"
            />
            <CommandList style={{ maxHeight }}>
              {loading ? (
                <div className="py-8 text-center text-sm text-slate-400">
                  {t("common.loading", {}, { fallback: "加载中..." })}
                </div>
              ) : filteredOptions.length === 0 ? (
                <CommandEmpty>
                  {emptyText ??
                    t("contacts.multiSelect.empty", {}, { fallback: "未找到联系人" })}
                </CommandEmpty>
              ) : (
                <CommandGroup>
                  {filteredOptions.map((contact) => {
                    const isSelected = value.includes(contact.id);
                    return (
                      <CommandItem
                        key={contact.id}
                        value={contact.id}
                        onSelect={() => toggleContact(contact.id)}
                        className={cn(
                          "flex items-center gap-2 px-3 py-2 cursor-pointer",
                          isSelected && "bg-slate-50"
                        )}
                      >
                        <div
                          className={cn(
                            "w-4 h-4 rounded border flex items-center justify-center transition-colors",
                            isSelected
                              ? "bg-blue-600 border-blue-600"
                              : "border-slate-300"
                          )}
                        >
                          {isSelected && <Check className="w-3 h-3 text-white" />}
                        </div>
                        <div className="w-7 h-7 rounded-full bg-slate-100 flex items-center justify-center shrink-0">
                          <User className="w-3.5 h-3.5 text-slate-500" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-slate-900 truncate">
                            {contact.name}
                          </p>
                          {contact.code && (
                            <p className="text-[10px] text-slate-400 font-mono">
                              {contact.code}
                            </p>
                          )}
                        </div>
                        {contact.type && (
                          <span className="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 text-slate-500">
                            {contact.type}
                          </span>
                        )}
                      </CommandItem>
                    );
                  })}
                </CommandGroup>
              )}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>

      {selectedContacts.length > 0 && (
        <p className="mt-1 text-xs text-slate-400">
          {t("contacts.multiSelect.selected", {}, { fallback: "已选择" })} {selectedContacts.length} {t("contacts.multiSelect.person", {}, { fallback: "人" })}
        </p>
      )}
    </div>
  );
}
