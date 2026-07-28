/**
 * EntityFormPage — 标准实体表单页面组件
 *
 * 对齐 MODULE_SPEC §9.2 标准骨架。
 * import { EntityFormPage } from "@alioth/composables";
 */

import * as React from "react";
import type { z } from "zod";
import { Button } from "@alioth/components";
import { useT } from "@alioth/i18n";
import { AutoForm } from "./AutoForm";
import type { AutoFormProps } from "./AutoForm";

export interface EntityFormPageProps<TSchema extends z.ZodType> {
  schema: TSchema;
  title: string;
  mode: "create" | "edit";
  onSubmit: (values: z.infer<TSchema>) => void | Promise<void>;
  onCancel?: () => void;
  defaultValues?: AutoFormProps<TSchema>["defaultValues"];
  fieldConfig?: AutoFormProps<TSchema>["fieldConfig"];
  groups?: AutoFormProps<TSchema>["groups"];
  isLoading?: boolean;
  error?: string | null;
  onRetry?: () => void;
  /** _refs 关联数据（编辑回填时 reference/scalar 字段显示 label） */
  refs?: Record<string, Record<string, unknown>>;
}

export function EntityFormPage<TSchema extends z.ZodType>({
  schema,
  title,
  mode,
  onSubmit,
  onCancel,
  defaultValues,
  fieldConfig,
  groups,
  isLoading = false,
  error = null,
  onRetry,
  refs,
}: EntityFormPageProps<TSchema>): React.ReactElement {
  const t = useT();

  if (error) {
    return (
      <div className="page">
        <div className="page-header">
          <h1>{title}</h1>
        </div>
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground flex-col gap-4">
          <p>{error}</p>
          {onRetry && <Button variant="outline" onClick={onRetry}>{t("common.retry") ?? "Retry"}</Button>}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="page">
        <div className="page-header">
          <h1>{title}</h1>
        </div>
        <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
          {t("common.loading") ?? "Loading..."}
        </div>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>{title}</h1>
      </div>
      <AutoForm
        schema={schema}
        onSubmit={onSubmit}
        defaultValues={defaultValues}
        fieldConfig={fieldConfig}
        groups={groups}
        isLoading={false}
        submitText={mode === "create" ? t("common.create") ?? "Create" : t("common.save") ?? "Save"}
        cancelText={onCancel ? t("common.cancel") ?? "Cancel" : undefined}
        onCancel={onCancel}
        refs={refs}
      />
    </div>
  );
}
