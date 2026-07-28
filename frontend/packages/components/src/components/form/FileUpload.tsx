/**
 * FileUpload · 文件上传组件
 *
 * 支持拖拽/点击选择文件，显示文件名和大小。
 */

import { useState, useRef, useCallback } from "react";
import { Upload, File, X } from "lucide-react";
import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";

export interface FileUploadProps {
  value: string | null;
  onChange: (value: string | null | File) => void;
  accept?: string;
  maxSize?: number;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
}

export function FileUpload({
  value,
  onChange,
  accept,
  maxSize,
  disabled,
  placeholder,
  className,
}: FileUploadProps) {
  const t = useT();
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFile = useCallback(
    (file: File) => {
      if (maxSize && file.size > maxSize) {
        import("sonner").then(({ toast }) => {
          toast.error(
            t("autoform.fileTooLarge", {}, { fallback: "文件过大" }) +
              `: ${(maxSize / 1024 / 1024).toFixed(1)}MB`,
          );
        });
        return;
      }
      setFileName(file.name);
      onChange(file);
    },
    [maxSize, onChange, t],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const file = e.dataTransfer.files?.[0];
      if (file) handleFile(file);
    },
    [handleFile],
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) handleFile(file);
      if (inputRef.current) inputRef.current.value = "";
    },
    [handleFile],
  );

  if (value || fileName) {
    return (
      <div
        className={cn(
          "flex items-center gap-2 rounded-lg border border-border bg-muted/30 px-3 py-2 text-sm",
          className,
        )}
      >
        <File className="w-4 h-4 text-muted-foreground shrink-0" />
        <span className="flex-1 truncate">{fileName ?? value}</span>
        {!disabled && (
          <button
            type="button"
            onClick={() => {
              setFileName(null);
              onChange(null);
            }}
            className="text-destructive hover:text-destructive/80 text-xs shrink-0"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        )}
      </div>
    );
  }

  return (
    <label
      className={cn(
        "flex flex-col items-center justify-center w-full h-24 rounded-lg border-2 border-dashed border-border bg-muted/20 cursor-pointer hover:bg-muted/40 transition-colors",
        dragOver && "border-primary bg-primary/5",
        disabled && "opacity-50 cursor-not-allowed",
        className,
      )}
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
    >
      <Upload className="w-6 h-6 text-muted-foreground mb-1" />
      <div className="text-xs text-muted-foreground text-center px-4">
        {placeholder ?? t("autoform.uploadHint", {}, { fallback: "点击或拖拽上传文件" })}
      </div>
      <input
        ref={inputRef}
        type="file"
        className="hidden"
        accept={accept}
        disabled={disabled}
        onChange={handleChange}
      />
    </label>
  );
}

FileUpload.displayName = "FileUpload";
