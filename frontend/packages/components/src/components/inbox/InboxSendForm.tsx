/**
 * InboxSendForm · 站内信发送表单
 *
 * 用于在右侧面板内发送新站内信。
 * 包含收件人、抄送人（多选联系人）、标题、正文输入。
 * 收件人和抄送人均使用 ContactMultiSelect 组件选择联系人。
 */

import * as React from "react";
import {
  ArrowLeft,
  Send,
  Users,
  Mail,
  PenLine,
} from "lucide-react";
import { cn } from "../../lib/utils";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Textarea } from "../ui/textarea";
import { Separator } from "../ui/separator";
import { Badge } from "../ui/badge";
import { useT } from "@alioth/i18n";
import { ContactMultiSelect } from "../form/ContactMultiSelect";
import type { InboxSendFormProps } from "./types";

export const InboxSendForm = React.forwardRef<
  HTMLDivElement,
  InboxSendFormProps
>(
  (
    {
      contacts,
      onBack,
      onSend,
      loading = false,
      className,
    },
    ref,
  ) => {
    const t = useT();
    const [toRecipients, setToRecipients] = React.useState<string[]>([]);
    const [ccRecipients, setCcRecipients] = React.useState<string[]>([]);
    const [subject, setSubject] = React.useState("");
    const [body, setBody] = React.useState("");
    const [showCc, setShowCc] = React.useState(false);

    const canSend =
      toRecipients.length > 0 && subject.trim().length > 0 && body.trim().length > 0;

    const handleSend = () => {
      if (!canSend) return;
      onSend?.({
        to: toRecipients,
        cc: ccRecipients,
        subject: subject.trim(),
        body: body.trim(),
      });
    };

    const handleBack = () => {
      setToRecipients([]);
      setCcRecipients([]);
      setSubject("");
      setBody("");
      setShowCc(false);
      onBack?.();
    };

    // 过滤掉已选收件人，避免重复
    const ccAvailableContacts = React.useMemo(
      () => contacts.filter((c) => !toRecipients.includes(c.id)),
      [contacts, toRecipients]
    );

    return (
      <div ref={ref} className={cn("flex flex-col h-full", className)}>
        {/* 头部：返回 + 发送 */}
        <div className="flex items-center justify-between px-6 py-3 border-b shrink-0">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleBack}
            className="gap-1 text-muted-foreground hover:text-foreground -gl-2"
          >
            <ArrowLeft className="w-4 h-4" />
            {t("components.inbox.backToList")}
          </Button>
          <Button
            size="sm"
            onClick={handleSend}
            disabled={!canSend || loading}
            className="gap-1"
          >
            <Send className="w-3.5 h-3.5" />
            {t("components.inbox.send", {}, { fallback: "发送" })}
          </Button>
        </div>

        {/* 表单内容 */}
        <div className="flex-1 overflow-y-auto px-6 py-5 space-y-5">
          {/* 收件人 */}
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <Users className="w-4 h-4 text-slate-500" />
              <span className="text-sm font-medium text-slate-700">
                {t("components.inbox.to", {}, { fallback: "收件人" })}
              </span>
              <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-4">
                {t("components.inbox.required", {}, { fallback: "必填" })}
              </Badge>
            </div>
            <ContactMultiSelect
              value={toRecipients}
              onChange={setToRecipients}
              options={contacts}
              placeholder={t(
                "components.inbox.selectRecipients",
                {},
                { fallback: "请选择收件人..." }
              )}
              searchPlaceholder={t(
                "components.inbox.searchRecipients",
                {},
                { fallback: "搜索联系人..." }
              )}
              emptyText={t(
                "components.inbox.noContacts",
                {},
                { fallback: "暂无可选联系人" }
              )}
            />
          </div>

          {/* 抄送人（可选，点击展开） */}
          <div>
            <button
              type="button"
              onClick={() => setShowCc((prev) => !prev)}
              className="flex items-center gap-2 text-sm text-slate-500 hover:text-slate-700 transition-colors mb-1.5"
            >
              <Mail className="w-4 h-4" />
              <span>
                {t("components.inbox.cc", {}, { fallback: "抄送人" })}
              </span>
              {!showCc && (
                <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-4 cursor-pointer">
                  {t("common.optional", {}, { fallback: "可选" })}
                </Badge>
              )}
            </button>

            {showCc && (
              <ContactMultiSelect
                value={ccRecipients}
                onChange={setCcRecipients}
                options={ccAvailableContacts}
                placeholder={t(
                  "components.inbox.selectCc",
                  {},
                  { fallback: "请选择抄送人..." }
                )}
                searchPlaceholder={t(
                  "components.inbox.searchCc",
                  {},
                  { fallback: "搜索抄送人..." }
                )}
                emptyText={t(
                  "components.inbox.noContacts",
                  {},
                  { fallback: "暂无可选联系人" }
                )}
              />
            )}
          </div>

          <Separator />

          {/* 标题 */}
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <PenLine className="w-4 h-4 text-slate-500" />
              <span className="text-sm font-medium text-slate-700">
                {t("components.inbox.subject", {}, { fallback: "主题" })}
              </span>
            </div>
            <Input
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              placeholder={t(
                "components.inbox.subjectPlaceholder",
                {},
                { fallback: "请输入消息主题..." }
              )}
              className="text-sm"
            />
          </div>

          {/* 正文 */}
          <div>
            <div className="flex items-center gap-2 mb-1.5">
              <Mail className="w-4 h-4 text-slate-500" />
              <span className="text-sm font-medium text-slate-700">
                {t("components.inbox.body", {}, { fallback: "正文" })}
              </span>
            </div>
            <Textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder={t(
                "components.inbox.bodyPlaceholder",
                {},
                { fallback: "请输入消息内容..." }
              )}
              className="min-h-[160px] text-sm resize-none"
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault();
                  handleSend();
                }
              }}
            />
            <p className="mt-1.5 text-xs text-slate-400">
              {t(
                "components.inbox.quickSendHint",
                {},
                { fallback: "Cmd/Ctrl + Enter 快速发送" }
              )}
            </p>
          </div>
        </div>
      </div>
    );
  }
);

InboxSendForm.displayName = "InboxSendForm";
