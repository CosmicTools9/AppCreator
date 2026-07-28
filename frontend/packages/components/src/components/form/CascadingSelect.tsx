/**
 * CascadingSelect · 多级级联选择（省/市/区）
 *
 * 每一级是一个独立的 SearchableSelect；选中上一级后，下一级的选项
 * 自动过滤为上一级选中项的 children。Value: string[]（每级一个 value）。
 */
import { cn } from "../../lib/utils";
import { SearchableSelect } from "./SearchableSelect";
import { useT } from "@alioth/i18n";

export interface CascadingOption {
  value: string;
  label: string;
  children?: CascadingOption[];
  disabled?: boolean;
}

export interface CascadingSelectProps {
  options: CascadingOption[];
  /** 当前选中值数组（每一级一个），如 ["110000","110100","110101"] */
  value: string[];
  onChange: (value: string[]) => void;
  /** 自定义每一级的占位符数组，长度决定默认渲染层级数量 */
  placeholders?: string[];
  className?: string;
  disabled?: boolean;
  /** 各级布局方式：horizontal（行内） | vertical（堆叠） */
  layout?: "horizontal" | "vertical";
}

function toFlatOptions(
  options: CascadingOption[],
): { value: string; label: string; disabled?: boolean }[] {
  return options.map((o) => ({
    value: o.value,
    label: o.label,
    disabled: o.disabled,
  }));
}

function findOption(
  options: CascadingOption[],
  value: string,
): CascadingOption | null {
  for (const opt of options) {
    if (opt.value === value) return opt;
    if (opt.children) {
      const found = findOption(opt.children, value);
      if (found) return found;
    }
  }
  return null;
}

export function CascadingSelect({
  options,
  value,
  onChange,
  placeholders,
  className,
  disabled,
  layout = "horizontal",
}: CascadingSelectProps) {
  const t = useT();
  const defaultPlaceholder = t("common.pleaseSelect", {}, { fallback: "请选择" });

  // 计算每一级的选项与标签
  const levelOptions: { value: string; label: string; disabled?: boolean }[][] = [];
  let current: CascadingOption[] | undefined = options;
  for (let i = 0; i < value.length + 1 && current; i++) {
    levelOptions.push(toFlatOptions(current));
    const parent = findOption(current, value[i]);
    current = parent?.children;
  }

  const handleLevelChange = (level: number) => (nextValue: string) => {
    const newValue = value.slice(0, level);
    newValue[level] = nextValue;
    onChange(newValue);
  };

  const visibleLevels = levelOptions.length;

  return (
    <div
      className={cn(
        "w-full",
        layout === "horizontal"
          ? "grid grid-flow-col auto-cols-fr gap-2"
          : "flex flex-col gap-2",
        className,
      )}
    >
      {Array.from({ length: visibleLevels }).map((_, level) => {
        const opts = levelOptions[level];
        const lvlValue = value[level] ?? "";
        const placeholder = placeholders?.[level] ?? defaultPlaceholder;
        return (
          <SearchableSelect
            key={level}
            options={opts}
            value={lvlValue}
            onChange={handleLevelChange(level)}
            placeholder={placeholder}
            disabled={disabled}
          />
        );
      })}
    </div>
  );
}

CascadingSelect.displayName = "CascadingSelect";