/**
 * AutoForm Component
 *
 * Zod Schema-driven form generation for AliothStudio.
 * Automatically renders form fields based on Zod schema definitions.
 */

import * as React from 'react';
import { z } from 'zod';
import { useForm, useWatch } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import type {
  UseFormReturn,
  DefaultValues,
  SubmitHandler,
  FieldValues,
} from 'react-hook-form';

import { useT } from "@alioth/i18n";
import { cn } from "@alioth/components";
import { Button } from "@alioth/components";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@alioth/components";
import { Input } from "@alioth/components";
import { Checkbox } from "@alioth/components";
import { Textarea } from "@alioth/components";
import { SearchableSelect } from "@alioth/components";
import { ReferenceSelect } from '@alioth/composables/form';
import { Switch } from "@alioth/components";
import { Popover, PopoverContent, PopoverTrigger } from "@alioth/components";
import {
  MultiSelect,
  TagInput,
  DateTimePicker,
  DateRangePicker,
  FileUpload,
  CascadingSelect,
  RichTextEditor,
  ScalarField,
} from "@alioth/components";
import {
  format,
  parseISO,
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  addDays,
  isSameMonth,
  isSameDay,
  addMonths,
  subMonths,
} from 'date-fns';
// import { zhCN } from 'date-fns/locale/zh-CN';
import { CalendarIcon, ChevronDown } from 'lucide-react';

// ============================================
// Types
// ============================================

/** Field type configuration */
export interface FieldConfig<TFieldValues extends FieldValues = FieldValues> {
  /** Custom label (defaults to field key) */
  label?: string;
  /** Field description/help text */
  description?: string;
  /** Placeholder text */
  placeholder?: string;
  /** Custom input component (receives field.value/onChange/onBlur and form directly) */
  component?: React.ComponentType<{
    value: unknown;
    onChange: (value: unknown) => void;
    onBlur: () => void;
    form: UseFormReturn<TFieldValues>;
    formValues?: Record<string, unknown>;
    [key: string]: unknown;
  }>;
  /** For select/enum fields */
  options?: { label: string; value: string | number }[];
  /** Field type override */
  type?:
    | "text"
    | "number"
    | "email"
    | "password"
    | "textarea"
    | "date"
    | "select"
    | "checkbox"
    | "switch"
    | "color"
    | "reference"
    | "multi-select"
    | "richtext"
    | "datetime"
    | "date-range"
    | "upload"
    | "tags"
    | "cascader"
    | "scalar"
    | "editor-json";
  /** Whether field is disabled */
  disabled?: boolean;
  /** Type `reference`: API endpoint path for fetching options */
  endpoint?: string;
  /** Type `reference`: field name to use as display label (e.g. "notice" / "code" / "name") */
  labelField?: string;
  /** Custom validation message */
  validationMessage?: string;
  /** Conditional visibility: show this field only when `field` matches the given condition */
  showIf?: { field: string; eq?: unknown; neq?: unknown; notEmpty?: boolean };
  /** Fields to watch for dependency changes (re-render / re-validate when these change) */
  watchFields?: string[];
  /** Number of columns within a group (1-4, for multi-column layout) */
  columns?: number;
  /** Whether field supports inline editing */
  inlineEdit?: boolean;
  /** For upload fields, accepted file types (e.g. "image/*", ".pdf,.doc") */
  accept?: string;
  /** For upload, max file size in bytes */
  maxSize?: number;
  /** For multi-select/upload, allow multiple values */
  multiple?: boolean;
  /** For scalar type, which subtype */
  scalarField?: "price" | "date" | "common";
  /** For scalar type, options for the unit dropdown */
  unitOptions?: { value: string; label: string }[];
}

/** AutoForm props */
export interface AutoFormProps<TSchema extends z.ZodType> {
  /** Zod schema for form validation */
  schema: TSchema;
  /** Form submission handler */
  onSubmit: (values: z.infer<TSchema>) => void | Promise<void>;
  /** Default values */
  defaultValues?: DefaultValues<z.infer<TSchema>>;
  /** Field configurations */
  fieldConfig?: Partial<Record<keyof z.infer<TSchema>, FieldConfig>>;
  /** Custom class name */
  className?: string;
  /** Submit button text */
  submitText?: string;
  /** Cancel button text (shows cancel button if provided) */
  cancelText?: string;
  /** Cancel handler */
  onCancel?: () => void;
  /** Loading state */
  isLoading?: boolean;
  /** Submit button variant */
  submitVariant?: React.ComponentProps<typeof Button>["variant"];
  /** Form layout */
  layout?: "vertical" | "horizontal" | "inline";
  /** Whether to show labels (reserved for future use) */
  showLabels?: boolean;
  /** Custom render function for form content */
  children?: (form: UseFormReturn<any>) => React.ReactNode;
  /** Hide built-in submit/cancel buttons */
  hideButtons?: boolean;
  /** Field groups for sectioned layout */
  groups?: Array<{ title: string; fields: string[] }>;
  /**
   * _refs 关联数据，用于编辑回填时 reference/scalar 字段显示 label 而非 raw ID。
   * 格式: { fk_subject: { notice: "供应商A" }, qk_price: { mark: 999, sk_unit: "元" } }
   */
  refs?: Record<string, Record<string, unknown>>;
}

// ============================================
// Schema Analysis
// ============================================

/**
 * Detects if a schema is a specific Zod type.
 * Uses constructor.name and _zod.traits to handle monorepo
 * multi-instance scenarios where instanceof may fail.
 */
function isZodType(schema: any, typeName: string): boolean {
  if (!schema) return false;
  if (schema.constructor?.name === typeName) return true;
  if (schema._zod?.traits?.has(typeName)) return true;
  return false;
}

/**
 * Analyzes Zod schema to extract field information
 */
function analyzeSchema<T extends z.ZodType>(
  schema: T,
): Array<{
  key: string;
  type: string;
  isOptional: boolean;
  isNullable: boolean;
  description?: string;
  enumValues?: string[];
}> {
  const shape = (schema as unknown as z.ZodObject<Record<string, z.ZodType>>)
    .shape;
  if (!shape) return [];

  return Object.entries(shape).map(([key, fieldSchema]) => {
    let type = "unknown";
    let isOptional = false;
    let isNullable = false;
    let description: string | undefined;
    let enumValues: string[] | undefined;

    let currentSchema: any = fieldSchema;

    // Unwrap optional/nullable layers (handles .optional().nullable() etc.)
    while (true) {
      if (isZodType(currentSchema, "ZodOptional")) {
        isOptional = true;
        currentSchema = currentSchema.unwrap();
      } else if (isZodType(currentSchema, "ZodNullable")) {
        isNullable = true;
        currentSchema = currentSchema.unwrap();
      } else {
        break;
      }
    }

    // Get description
    if (currentSchema.description) {
      description = currentSchema.description;
    }

    // Determine type
    if (isZodType(currentSchema, "ZodString")) {
      type = "string";
    } else if (isZodType(currentSchema, "ZodNumber")) {
      type = "number";
    } else if (isZodType(currentSchema, "ZodBoolean")) {
      type = "boolean";
    } else if (isZodType(currentSchema, "ZodDate")) {
      type = "date";
    } else if (isZodType(currentSchema, "ZodEnum")) {
      type = "enum";
      enumValues = (currentSchema as any).options as string[];
    } else if (isZodType(currentSchema, "ZodArray")) {
      type = "array";
    } else if (isZodType(currentSchema, "ZodObject")) {
      type = "object";
    }

    return { key, type, isOptional, isNullable, description, enumValues };
  });
}

// ============================================
// Custom Calendar (matches Ant Design dark style)
// ============================================

function CustomCalendar({
  selected,
  onSelect,
}: {
  selected?: Date;
  onSelect: (date: Date | null) => void;
}) {
  const [currentMonth, setCurrentMonth] = React.useState(selected || new Date());

  const monthStart = startOfMonth(currentMonth);
  const monthEnd = endOfMonth(monthStart);
  const calendarStart = startOfWeek(monthStart, { weekStartsOn: 0 });
  const calendarEnd = endOfWeek(monthEnd, { weekStartsOn: 0 });

  const days: Date[] = [];
  let day = calendarStart;
  while (day <= calendarEnd) {
    days.push(day);
    day = addDays(day, 1);
  }

  const weekDays = ['日', '一', '二', '三', '四', '五', '六'];

  return (
    <div className="bg-[#1f1f1f] rounded-md p-3 w-[272px]">
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <div className="text-white text-sm font-medium flex items-center">
          {format(currentMonth, "yyyy年MM月")}
          <ChevronDown className="h-3 w-3 gl-1 opacity-50" />
        </div>
        <div className="flex items-center gap-0.5">
          <button
            type="button"
            className="h-6 w-6 flex items-center justify-center text-white/50 hover:text-white text-xs"
            onClick={() => setCurrentMonth(subMonths(currentMonth, 1))}
          >
            ↑
          </button>
          <button
            type="button"
            className="h-6 w-6 flex items-center justify-center text-white/50 hover:text-white text-xs"
            onClick={() => setCurrentMonth(addMonths(currentMonth, 1))}
          >
            ↓
          </button>
        </div>
      </div>

      {/* Week days */}
      <div className="grid grid-cols-7 mb-1">
        {weekDays.map((d) => (
          <div key={d} className="h-7 flex items-center justify-center text-xs text-white/40">
            {d}
          </div>
        ))}
      </div>

      {/* Days */}
      <div className="grid grid-cols-7 gap-y-0.5">
        {days.map((d, i) => {
          const isCurrentMonth = isSameMonth(d, currentMonth);
          const isSelected = selected && isSameDay(d, selected);
          const isToday = isSameDay(d, new Date());

          return (
            <button
              key={i}
              type="button"
              className={cn(
                "h-8 w-8 flex items-center justify-center text-sm rounded transition-colors mx-auto",
                !isCurrentMonth && "text-white/25",
                isCurrentMonth && !isSelected && "text-white/85 hover:bg-white/10",
                isSelected && "bg-[#1677ff] text-white",
                isToday && !isSelected && "text-[#1677ff] border border-[#1677ff]/40",
              )}
              onClick={() => onSelect(d)}
            >
              {format(d, "d")}
            </button>
          );
        })}
      </div>

      {/* Footer */}
      <div className="flex justify-between items-center mt-2 pt-2 border-t border-white/10">
        <button
          type="button"
          className="text-xs text-[#1677ff] hover:opacity-80 px-1 py-0.5"
          onClick={() => onSelect(null)}
        >
          清除
        </button>
        <button
          type="button"
          className="text-xs text-[#1677ff] hover:opacity-80 px-1 py-0.5"
          onClick={() => onSelect(new Date())}
        >
          今天
        </button>
      </div>
    </div>
  );
}

// ============================================
// Field Renderer
// ============================================

/**
 * Renders an individual form field based on schema type
 */
function AutoFormField({
  name,
  schemaInfo,
  fieldConfig,
  form,
  showLabels = true,
  refs,
}: {
  name: string;
  schemaInfo: ReturnType<typeof analyzeSchema>[number];
  fieldConfig?: FieldConfig;
  form: UseFormReturn<any>;
  showLabels?: boolean;
  refs?: Record<string, Record<string, unknown>>;
}) {
  const t = useT();
  const { type, isOptional, description, enumValues } = schemaInfo;
  const config = fieldConfig || {};
  const label = config.label || name;
  const fieldDescription = config.description || description;

  // showIf: conditional visibility
  if (config.showIf) {
    const watchedValue = useWatch({ control: form.control, name: config.showIf.field });
    let visible = true;
    if (config.showIf.eq !== undefined && watchedValue !== config.showIf.eq) {
      visible = false;
    }
    if (config.showIf.neq !== undefined && watchedValue === config.showIf.neq) {
      visible = false;
    }
    if (config.showIf.notEmpty && !watchedValue) {
      visible = false;
    }
    if (!visible) return null;
  }

  // Determine component type
  const componentType =
    config.type || getDefaultComponentType(type, enumValues);

  // Get _refs label for reference fields (edit mode display)
  const refData = refs?.[name];
  const refLabel = refData
    ? String(refData.notice ?? refData.name ?? refData.code ?? refData.mark ?? "")
    : undefined;

  return (
    <FormField
      control={form.control}
      name={name}
      render={({ field }) => (
        <FormItem>
          {showLabels && componentType !== "checkbox" && componentType !== "switch" && (
            <FormLabel>
              {label}
              {!isOptional && <span className="text-destructive gl-1">*</span>}
            </FormLabel>
          )}
          <FormControl>
            {config.component ? (
              <config.component
                value={field.value}
                onChange={field.onChange}
                onBlur={field.onBlur}
                form={form}
                formValues={form.getValues()}
              />
            ) : (
              renderFieldByType(
                componentType,
                field,
                config as FieldConfig,
                enumValues,
                t,
                refLabel,
              )
            )}
          </FormControl>
          {showLabels && (componentType === "checkbox" || componentType === "switch") && (
            <FormLabel className="font-normal gl-2">
              {label}
              {!isOptional && <span className="text-destructive gl-1">*</span>}
            </FormLabel>
          )}
          {fieldDescription && (
            <FormDescription>{fieldDescription}</FormDescription>
          )}
          <FormMessage />
        </FormItem>
      )}
    />
  );
}

/**
 * Gets default component type from schema type
 */
function getDefaultComponentType(
  schemaType: string,
  enumValues?: string[],
): FieldConfig["type"] {
  if (enumValues) return "select";

  switch (schemaType) {
    case "string":
      return "text";
    case "number":
      return "number";
    case "boolean":
      return "checkbox";
    case "date":
      return "date";
    case "array":
      return "multi-select";
    case "object":
      return "text";
    default:
      return "text";
  }
}

/**
 * Renders the appropriate input component based on type
 */
function renderFieldByType(
  type: FieldConfig["type"],
  field: {
    name: string;
    value: unknown;
    onChange: (value: unknown) => void;
    onBlur: () => void;
  },
  config: FieldConfig,
  enumValues: string[] | undefined,
  t: ReturnType<typeof useT>,
  refLabel?: string,
): React.ReactElement {
  const placeholder = config.placeholder;
  const disabled = config.disabled;

  switch (type) {
    case "textarea":
      return (
        <Textarea
          placeholder={placeholder}
          disabled={disabled}
          value={(field.value as string) || ""}
          onChange={field.onChange}
          onBlur={field.onBlur}
        />
      );

    case "select": {
      const selectOptions =
        config.options ||
        enumValues?.map((v) => ({ label: v, value: v })) ||
        [];
      return (
        <SearchableSelect
          options={selectOptions.map((opt) => ({
            label: opt.label,
            value: String(opt.value),
          }))}
          value={field.value != null ? String(field.value) : ""}
          onChange={(v) => {
            // 如果原字段值为数字类型，转回数字；否则保持字符串
            const originalValue = field.value;
            const num = Number(v);
            if (typeof originalValue === "number" || (originalValue == null && !Number.isNaN(num) && v !== "")) {
              field.onChange(Number.isNaN(num) ? undefined : num);
            } else {
              field.onChange(v || undefined);
            }
          }}
          placeholder={placeholder || t("common.pleaseSelect")}
          disabled={disabled}
        />
      );
    }

    case "checkbox":
      return (
        <Checkbox
          checked={(field.value as boolean) || false}
          onCheckedChange={field.onChange}
          disabled={disabled}
        />
      );

    case "switch":
      return (
        <Switch
          checked={(field.value as boolean) || false}
          onCheckedChange={field.onChange}
          disabled={disabled}
        />
      );

    case "date": {
      const dateValue = field.value
        ? (field.value instanceof Date ? field.value : parseISO(field.value as string))
        : undefined;
      return (
        <Popover>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              className={cn(
                "w-full justify-start text-left font-normal",
                !field.value && "text-muted-foreground",
              )}
              disabled={disabled}
            >
              <CalendarIcon className="mr-2 h-4 w-4" />
              {field.value
                ? format(dateValue as Date, "yyyy/MM/dd")
                : placeholder || t("autoform.pickDate")}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-auto p-0 border-0 bg-transparent shadow-xl">
            <CustomCalendar
              selected={dateValue}
              onSelect={(date) => field.onChange(date ? format(date, "yyyy-MM-dd") : null)}
            />
          </PopoverContent>
        </Popover>
      );
    }

    case "number":
      return (
        <Input
          type="number"
          name={field.name}
          autoComplete="off"
          placeholder={placeholder}
          disabled={disabled}
          value={(field.value as number) ?? ""}
          onChange={(e) => field.onChange(e.target.value === '' ? undefined : e.target.valueAsNumber)}
          onBlur={field.onBlur}
        />
      );

    case "email":
      return (
        <Input
          type="email"
          name={field.name}
          autoComplete="email"
          spellCheck={false}
          placeholder={placeholder}
          disabled={disabled}
          value={(field.value as string) || ""}
          onChange={field.onChange}
          onBlur={field.onBlur}
        />
      );

    case "password":
      return (
        <Input
          type="password"
          name={field.name}
          autoComplete="new-password"
          spellCheck={false}
          placeholder={placeholder}
          disabled={disabled}
          value={(field.value as string) || ""}
          onChange={field.onChange}
          onBlur={field.onBlur}
        />
      );

    case "color": {
      const presetColors = [
        "#3B82F6", "#EF4444", "#22C55E", "#EAB308",
        "#A855F7", "#F97316", "#06B6D4", "#6B7280",
      ];
      return (
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={(field.value as string) || "#000000"}
              onChange={(e) => field.onChange(e.target.value)}
              className="h-9 w-9 rounded border p-0 cursor-pointer shrink-0"
            />
            <Input
              type="text"
              value={(field.value as string) || ""}
              onChange={field.onChange}
              onBlur={field.onBlur}
              placeholder={placeholder || "#3B82F6"}
              className="flex-1"
            />
          </div>
          <div className="flex items-center gap-2 flex-wrap">
            {presetColors.map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => field.onChange(c)}
                className={cn(
                  "h-6 w-6 rounded-full border-2 transition-all",
                  field.value === c ? "border-foreground scale-110" : "border-transparent hover:scale-105"
                )}
                style={{ backgroundColor: c }}
                aria-label={`选择颜色 ${c}`}
              />
            ))}
          </div>
        </div>
      );
    }

    case "reference": {
      return (
        <ReferenceSelect
          endpoint={config.endpoint || ""}
          labelField={config.labelField || "notice"}
          value={field.value as string | number | null | undefined}
          onChange={field.onChange}
          placeholder={placeholder}
          disabled={disabled}
          initialLabel={refLabel}
        />
      );
    }

    case "multi-select": {
      const selectOptions =
        config.options ||
        enumValues?.map((v) => ({ label: v, value: v })) ||
        [];
      return (
        <MultiSelect
          options={selectOptions.map((opt) => ({
            value: String(opt.value),
            label: opt.label,
          }))}
          value={(field.value as string[]) || []}
          onChange={(v) => field.onChange(v)}
          placeholder={placeholder}
          disabled={disabled}
        />
      );
    }

    case "tags": {
      return (
        <TagInput
          value={(field.value as string[]) || []}
          onChange={(v) => field.onChange(v)}
          placeholder={placeholder}
          disabled={disabled}
        />
      );
    }

    case "datetime": {
      return (
        <DateTimePicker
          value={(field.value as string | null | undefined) ?? null}
          onChange={(v) => field.onChange(v)}
          placeholder={placeholder}
          disabled={disabled}
        />
      );
    }

    case "date-range": {
      const range = (field.value as { start?: string | null; end?: string | null }) || {};
      return (
        <DateRangePicker
          value={{ start: range.start ?? null, end: range.end ?? null }}
          onChange={(v) => field.onChange(v)}
          disabled={disabled}
        />
      );
    }

    case "upload": {
      return (
        <FileUpload
          value={(field.value as string | null) ?? null}
          onChange={(v) => field.onChange(v)}
          accept={config.accept}
          disabled={disabled}
        />
      );
    }

    case "richtext": {
      return (
        <RichTextEditor
          value={(field.value as string) || ""}
          onChange={(html) => field.onChange(html)}
          placeholder={placeholder}
          disabled={disabled}
        />
      );
    }
    case "cascader": {
      const cascaderOptions =
        (config.options as Array<{ value: string; label: string; children?: unknown[] }>) ||
        [];
      return (
        <CascadingSelect
          options={cascaderOptions as any}
          value={(field.value as string[]) || []}
          onChange={(v) => field.onChange(v)}
          placeholders={placeholder ? [placeholder] : undefined}
          disabled={disabled}
        />
      );
    }
    case "scalar": {
      const raw = (field.value as Record<string, unknown>) || {};
      const scalarField = config.scalarField || "common";
      if (scalarField === "date") {
        const dateVal = typeof (raw as { value?: string }).value === "string" ? (raw as { value?: string }).value : "";
        return (
          <div className="flex items-center gap-2">
            <Input
              type="date"
              value={dateVal}
              onChange={(e) => field.onChange({ value: e.target.value || undefined })}
              disabled={disabled}
              className="flex-1"
            />
          </div>
        );
      }
      return (
        <ScalarField
          value={raw as Record<string, unknown> | null | undefined}
          onChange={(v) => field.onChange(v)}
          scalarField={scalarField}
          disabled={disabled}
          placeholder={placeholder}
        />
      );
    }

    case "editor-json": {
      return (
        <Textarea
          value={(field.value as string) || ""}
          onChange={field.onChange}
          onBlur={field.onBlur}
          placeholder={placeholder || "{}"}
          disabled={disabled}
          className="font-mono text-xs"
        />
      );
    }

    case "text":
    default:
      return (
        <Input
          type="text"
          name={field.name}
          autoComplete="on"
          placeholder={placeholder}
          disabled={disabled}
          value={(field.value as string) || ""}
          onChange={field.onChange}
          onBlur={field.onBlur}
        />
      );
  }
}

// ============================================
// Main Component
// ============================================

/**
 * AutoForm - Zod Schema-driven form component
 *
 * @example
 * ```tsx
 * const userSchema = z.object({
 *   name: z.string().min(2),
 *   email: z.string().email(),
 *   role: z.enum(["admin", "user"]),
 * });
 *
 * <AutoForm
 *   schema={userSchema}
 *   onSubmit={(data) => handleSubmit(data)}}
 *   fieldConfig={{
 *     role: { type: "select", options: [{ label: "Admin", value: "admin" }] },
 *   }}
 * />
 * ```
 */
export function AutoForm<TSchema extends z.ZodType>({
  schema,
  onSubmit,
  defaultValues,
  fieldConfig = {},
  className,
  submitText,
  cancelText,
  onCancel,
  isLoading = false,
  submitVariant = "default",
  layout = "vertical",
  showLabels = true,
  children,
  hideButtons = false,
  groups,
  refs,
}: AutoFormProps<TSchema>) {
  // showLabels is passed down to AutoFormField below

  const t = useT();
  const effectiveSubmitText = submitText ?? t("common.submit", {}, { fallback: "提交" });
  const effectiveSavingText = t("common.saving", {}, { fallback: "保存中..." });

  const form = useForm({
    resolver: zodResolver(schema as any),
    defaultValues,
  });

  // 当 defaultValues 实质性变化时重置表单（用于异步数据加载完成后填充编辑表单）
  const defaultValuesRef = React.useRef<string | null>(null);
  React.useEffect(() => {
    if (!defaultValues) return;
    const serialized = JSON.stringify(defaultValues);
    if (defaultValuesRef.current !== serialized) {
      defaultValuesRef.current = serialized;
      form.reset(defaultValues);
    }
  }, [defaultValues, form]);

  const schemaFields = analyzeSchema(schema);

  const handleSubmit = async (values: any) => {
    await onSubmit(values as z.infer<TSchema>);
  };

  const handleInvalid = () => {
    const errors = form.formState.errors;
    const firstError = Object.values(errors)[0] as { message?: unknown } | undefined;
    if (firstError?.message) {
      import('sonner').then(({ toast }) => {
        toast.error(String(firstError.message));
      });
    }
  };

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(handleSubmit as SubmitHandler<any>, handleInvalid)}
        className={cn(
          "space-y-4",
          layout === "horizontal" && "grid grid-cols-2 gap-4 items-start",
          layout === "inline" && "flex flex-row items-end gap-4",
          className,
        )}
      >
        {children ? (
          children(form)
        ) : groups ? (
          <div className="space-y-6">
            {groups.map((group, idx) => {
              // 计算该组内字段的最高 columns 值（用于多列布局）
              const maxCols = Math.max(
                1,
                ...group.fields.map((k) => fieldConfig[k as keyof z.infer<TSchema>]?.columns ?? 1)
              );
              return (
                <div key={group.title + idx} className="space-y-4">
                  <h3 className="text-sm font-semibold text-foreground border-b pb-2">
                    {group.title}
                  </h3>
                  <div className={cn(
                    maxCols > 1 ? `grid grid-cols-${Math.min(maxCols, 4)} gap-4 items-start` : "space-y-4",
                    layout === "horizontal" && "grid grid-cols-2 gap-4 items-start"
                  )}>
                    {group.fields.map((fieldKey) => {
                      const field = schemaFields.find((f) => f.key === fieldKey);
                      if (!field) return null;
                      return (
                        <AutoFormField
                          key={field.key}
                          name={field.key}
                          schemaInfo={field}
                          fieldConfig={fieldConfig[field.key as keyof z.infer<TSchema>]}
                          form={form}
                          showLabels={showLabels}
                          refs={refs}
                        />
                      );
                    })}
                  </div>
                </div>
              );
            })}
            {/* Ungrouped fields */}
            {(() => {
              const groupedKeys = new Set(groups.flatMap((g) => g.fields));
              const ungrouped = schemaFields.filter((f) => !groupedKeys.has(f.key));
              if (ungrouped.length === 0) return null;
              return (
                <div className="space-y-4">
                  {ungrouped.map((field) => (
                    <AutoFormField
                      key={field.key}
                      name={field.key}
                      schemaInfo={field}
                      fieldConfig={fieldConfig[field.key as keyof z.infer<TSchema>]}
                      form={form}
                      showLabels={showLabels}
                      refs={refs}
                    />
                  ))}
                </div>
              );
            })()}
          </div>
        ) : (
          <>
            {schemaFields.map((field) => (
              <AutoFormField
                key={field.key}
                name={field.key}
                schemaInfo={field}
                fieldConfig={fieldConfig[field.key as keyof z.infer<TSchema>]}
                form={form}
                showLabels={showLabels}
                refs={refs}
              />
            ))}
          </>
        )}

        {!hideButtons && (
          <div className={cn(
            "flex gap-3",
            layout === "inline" && "gl-auto",
            layout === "horizontal" && "col-span-2 justify-start pt-4 border-t mt-2"
          )}>
            {cancelText && (
              <Button
                type="button"
                variant="outline"
                onClick={onCancel}
                disabled={isLoading}
              >
                {cancelText}
              </Button>
            )}
            <Button type="submit" variant={submitVariant} disabled={isLoading}>
              {isLoading ? effectiveSavingText : effectiveSubmitText}
            </Button>
          </div>
        )}
      </form>
    </Form>
  );
}
