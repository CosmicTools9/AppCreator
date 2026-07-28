/**
 * RichTextEditor · 轻量富文本编辑器
 *
 * 基于 contentEditable + document.execCommand（deprecated 但兼容性最好）。
 * 工具栏：Bold、Italic、Underline、Strike、列表（有序/无序）、H1/H2、清除格式。
 * Value: HTML string.
 */

import * as React from "react";
import {
  Bold,
  Italic,
  Underline,
  List,
  ListOrdered,
  Heading1,
  Heading2,
  Strikethrough,
  Eraser,
} from "lucide-react";

import { cn } from "../../lib/utils";
import { useT } from "@alioth/i18n";

export interface RichTextEditorProps {
  value: string;
  onChange: (html: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  /** 编辑器内容区最小高度 */
  minHeight?: number;
}

function exec(command: string, value?: string) {
  // execCommand 已 deprecated，但仍是目前最稳的轻量富文本方案。
  try {
    document.execCommand(command, false, value);
  } catch {
    // ignore — 某些浏览器对某些命令会抛错
  }
}

export function RichTextEditor({
  value,
  onChange,
  placeholder,
  disabled,
  className,
  minHeight = 160,
}: RichTextEditorProps) {
  const t = useT();
  const effectivePlaceholder =
    placeholder ?? t("form.editor.placeholder", {}, { fallback: "在此输入内容..." });

  const editorRef = React.useRef<HTMLDivElement>(null);
  const isInternalUpdate = React.useRef(false);

  // 同步外部 value 到编辑器（仅在外部 value 变化时）
  React.useEffect(() => {
    if (!editorRef.current) return;
    if (isInternalUpdate.current) {
      isInternalUpdate.current = false;
      return;
    }
    if (editorRef.current.innerHTML !== value) {
      editorRef.current.innerHTML = value || "";
    }
  }, [value]);

  const handleInput = () => {
    if (!editorRef.current) return;
    isInternalUpdate.current = true;
    onChange(editorRef.current.innerHTML);
  };

  const run = (cmd: string, val?: string) => {
    if (disabled) return;
    editorRef.current?.focus();
    exec(cmd, val);
    handleInput();
  };

  const buttons: Array<{
    key: string;
    icon: React.ReactNode;
    label: string;
    cmd: string;
    value?: string;
  }> = [
    { key: "bold", icon: <Bold className="h-3.5 w-3.5" />, label: "Bold", cmd: "bold" },
    { key: "italic", icon: <Italic className="h-3.5 w-3.5" />, label: "Italic", cmd: "italic" },
    { key: "underline", icon: <Underline className="h-3.5 w-3.5" />, label: "Underline", cmd: "underline" },
    { key: "strike", icon: <Strikethrough className="h-3.5 w-3.5" />, label: "Strikethrough", cmd: "strikeThrough" },
    { key: "h1", icon: <Heading1 className="h-3.5 w-3.5" />, label: "Heading 1", cmd: "formatBlock", value: "H1" },
    { key: "h2", icon: <Heading2 className="h-3.5 w-3.5" />, label: "Heading 2", cmd: "formatBlock", value: "H2" },
    { key: "ul", icon: <List className="h-3.5 w-3.5" />, label: "Bulleted list", cmd: "insertUnorderedList" },
    { key: "ol", icon: <ListOrdered className="h-3.5 w-3.5" />, label: "Numbered list", cmd: "insertOrderedList" },
    { key: "clear", icon: <Eraser className="h-3.5 w-3.5" />, label: "Clear formatting", cmd: "removeFormat" },
  ];

  const isEmpty = !value || value.replace(/<[^>]+>/g, "").trim().length === 0;

  return (
    <div
      className={cn(
        "w-full rounded-md border border-input bg-transparent shadow-sm overflow-hidden",
        disabled && "opacity-50 pointer-events-none",
        className,
      )}
    >
      <div className="flex flex-wrap items-center gap-0.5 border-b bg-muted/40 px-2 py-1.5">
        {buttons.map((b) => (
          <button
            key={b.key}
            type="button"
            title={b.label}
            aria-label={b.label}
            disabled={disabled}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => run(b.cmd, b.value)}
            className="h-7 w-7 inline-flex items-center justify-center rounded hover:bg-accent hover:text-accent-foreground text-muted-foreground transition-colors disabled:cursor-not-allowed"
          >
            {b.icon}
          </button>
        ))}
      </div>
      <div
        ref={editorRef}
        contentEditable={!disabled}
        suppressContentEditableWarning
        onInput={handleInput}
        onBlur={handleInput}
        data-placeholder={effectivePlaceholder}
        className={cn(
          "prose prose-sm max-w-none px-3 py-2 text-sm outline-none focus:outline-none",
          "min-h-[var(--editor-min-height)] overflow-auto",
          isEmpty && "before:content-[attr(data-placeholder)] before:text-muted-foreground before:pointer-events-none before:float-left before:h-0",
        )}
        style={{ ["--editor-min-height" as string]: `${minHeight}px` }}
      />
    </div>
  );
}

RichTextEditor.displayName = "RichTextEditor";