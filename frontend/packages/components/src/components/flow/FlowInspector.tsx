/**
 * FlowInspector — simplified node property inspector for FlowDesigner.
 *
 * Extracted from Alioth system-approve module FlowNodeInspector.
 * Covers approver/role assignment, SLA, sign mode, condition/cc/parallel/subflow/end settings.
 * Form schema and rule builder provideable via renderFormSchema/renderRuleBuilder render props.
 *
 * Usage:
 *   <FlowDesigner renderInspector={(props) =>
 *     <FlowInspector {...props} employees={employees} roles={roles} />
 *   } />
 */

import * as React from "react";
import { ApproverPicker, type ApproverRef } from "../approval/ApproverPicker";
import type { FlowNode } from "./types";

export interface FlowInspectorLabels {
  nodeName?: string;
  approverOrRole?: string;
  approverRole?: string;
  approverEngineer?: string;
  selectPlaceholder?: string;
  searchPlaceholder?: string;
  noApproverResults?: string;
  slaHours?: string;
  signMode?: string;
  signModeOr?: string;
  signModeAnd?: string;
  signModeSeq?: string;
  conditionRule?: string;
  ccRecipients?: string;
  ccPlaceholder?: string;
  parallelBranches?: string;
  subflowTarget?: string;
  browse?: string;
  endStatus?: string;
  endStatusComplete?: string;
  endStatusRejected?: string;
  endStatusCancelled?: string;
}

const DEFAULT_LABELS: FlowInspectorLabels = {
  nodeName: "Node Name",
  approverOrRole: "Approver / Role",
  approverRole: "Roles",
  approverEngineer: "Engineers",
  selectPlaceholder: "Search…",
  searchPlaceholder: "Search by name",
  noApproverResults: "No results",
  slaHours: "SLA (hours)",
  signMode: "Sign Mode",
  signModeOr: "Or-Sign",
  signModeAnd: "And-Sign",
  signModeSeq: "Sequential",
  conditionRule: "Condition Rule",
  ccRecipients: "CC Recipients",
  ccPlaceholder: "e.g. role:HR, user:123",
  parallelBranches: "Parallel Branches",
  subflowTarget: "Target Flow",
  browse: "Browse",
  endStatus: "End Status",
  endStatusComplete: "Complete",
  endStatusRejected: "Rejected",
  endStatusCancelled: "Cancelled",
};

function toApproverRef(node: FlowNode): ApproverRef | undefined {
  if (!node.role) return undefined;
  const kind = node.roleKind;
  if (kind === "engineer" || kind === "role") return { kind, id: node.role, label: node.role };
  return { kind: "role", id: node.role, label: node.role };
}

export interface FlowInspectorProps {
  node: FlowNode;
  onUpdate: (patch: Partial<FlowNode>) => void;
  /** Engineers available for assignment (from identity service) */
  employees: Array<{ id: number; name: string }>;
  /** Roles available for assignment (from identity service) */
  roles: Array<{ id: number; name: string }>;
  /** Labels for i18n */
  labels?: FlowInspectorLabels;
  /** Callback to enter a subflow (only for subflow nodes) */
  onEnterSubflow?: (targetCode: string) => void;
  /** Form schema editor render prop (optional, rendered in approval section if provided) */
  renderFormSchema?: () => React.ReactNode;
  /** Rule builder render prop (optional, replaces raw condition input if provided) */
  renderRuleBuilder?: (props: { value: string; onChange: (serialized: string) => void }) => React.ReactNode;
}

export function FlowInspector({
  node,
  onUpdate,
  employees,
  roles,
  labels: rawLabels,
  onEnterSubflow,
  renderFormSchema,
  renderRuleBuilder,
}: FlowInspectorProps) {
  const t = { ...DEFAULT_LABELS, ...rawLabels };

  const approverLabels = {
    roleTab: t.approverRole!,
    engineerTab: t.approverEngineer!,
    selectPlaceholder: t.selectPlaceholder!,
    searchPlaceholder: t.searchPlaceholder!,
    emptyText: t.noApproverResults!,
  };

  return React.createElement(
    "div",
    null,

    // Node name (all node types)
    React.createElement(
      "div",
      { className: "vfd-drawer-field", key: "name" },
      React.createElement("label", null, t.nodeName),
      React.createElement("input", {
        type: "text",
        value: node.label,
        onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ label: e.target.value }),
        className: "input",
        placeholder: node.type,
      }),
    ),

    // Approval node fields
    node.type === "approval" &&
      React.createElement(
        React.Fragment,
        { key: "approval" },
        React.createElement(
          "div",
          { className: "vfd-drawer-field" },
          React.createElement("label", null, t.approverOrRole),
          React.createElement(ApproverPicker, {
            value: toApproverRef(node),
            onChange: (ref: ApproverRef) => onUpdate({ role: ref.id, roleKind: ref.kind }),
            roles: roles.map((r) => ({ id: r.id, name: r.name })),
            engineers: employees.map((e) => ({ id: e.id, name: e.name })),
            labels: approverLabels,
          }),
        ),
        React.createElement(
          "div",
          { className: "vfd-drawer-field" },
          React.createElement("label", null, t.slaHours),
          React.createElement("input", {
            type: "number",
            value: node.sla ?? 24,
            min: 1,
            onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ sla: Number(e.target.value) }),
            className: "input",
          }),
        ),
        React.createElement(
          "div",
          { className: "vfd-drawer-field" },
          React.createElement("label", null, t.signMode),
          React.createElement(
            "select",
            {
              value: node.mode ?? "or_sign",
              onChange: (e: React.ChangeEvent<HTMLSelectElement>) => onUpdate({ mode: e.target.value }),
              className: "input",
            },
            React.createElement("option", { value: "or_sign" }, t.signModeOr),
            React.createElement("option", { value: "and_sign" }, t.signModeAnd),
            React.createElement("option", { value: "sequential" }, t.signModeSeq),
          ),
        ),
        renderFormSchema && React.createElement(
          "div",
          { className: "vfd-drawer-section", key: "form-schema" },
          React.createElement("div", { style: { fontSize: 12, fontWeight: 600, marginBottom: 8, color: "hsl(var(--muted-foreground))" } }, "Form Schema"),
          React.createElement("div", null, renderFormSchema()),
        ),
      ),

    // Condition node — uses renderRuleBuilder if provided, else raw expr input
    node.type === "condition" && (renderRuleBuilder
      ? React.createElement(
          "div",
          { key: "condition-slot" },
          renderRuleBuilder({ value: node.expr ?? "", onChange: (expr: string) => onUpdate({ expr }) }),
        )
      : React.createElement(
          "div",
          { className: "vfd-drawer-field", key: "condition" },
          React.createElement("label", null, t.conditionRule),
          React.createElement("input", {
            type: "text",
            value: node.expr ?? "",
            onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ expr: e.target.value }),
            className: "input",
          }),
        )
    ),

    // CC node
    node.type === "cc" &&
      React.createElement(
        "div",
        { className: "vfd-drawer-field", key: "cc" },
        React.createElement("label", null, t.ccRecipients),
        React.createElement("input", {
          type: "text",
          value: node.recipients ?? "",
          onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ recipients: e.target.value }),
          className: "input",
          placeholder: t.ccPlaceholder,
        }),
      ),

    // Parallel node
    node.type === "parallel" &&
      React.createElement(
        "div",
        { className: "vfd-drawer-field", key: "parallel" },
        React.createElement("label", null, t.parallelBranches),
        React.createElement("input", {
          type: "number",
          value: node.branches ?? 2,
          min: 2,
          max: 5,
          onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ branches: Number(e.target.value) }),
          className: "input",
        }),
      ),

    // Subflow node
    node.type === "subflow" &&
      React.createElement(
        "div",
        { className: "vfd-drawer-field", key: "subflow" },
        React.createElement("label", null, t.subflowTarget),
        React.createElement(
          "div",
          { style: { display: "flex", gap: 4 } },
          React.createElement("input", {
            type: "text",
            value: node.target ?? "",
            onChange: (e: React.ChangeEvent<HTMLInputElement>) => onUpdate({ target: e.target.value }),
            className: "input",
            placeholder: "AF-PROC-001",
          }),
          onEnterSubflow && node.target &&
            React.createElement(
              "button",
              {
                onClick: () => onEnterSubflow(node.target!),
                className: "btn btn-ghost btn-xs",
                style: { padding: "2px 6px", fontSize: 11 },
              },
              t.browse,
            ),
        ),
      ),

    // End node
    node.type === "end" &&
      React.createElement(
        "div",
        { className: "vfd-drawer-field", key: "end" },
        React.createElement("label", null, t.endStatus),
        React.createElement(
          "select",
          {
            value: node.outcome ?? "complete",
            onChange: (e: React.ChangeEvent<HTMLSelectElement>) => onUpdate({ outcome: e.target.value }),
            className: "input",
          },
          React.createElement("option", { value: "complete" }, t.endStatusComplete),
          React.createElement("option", { value: "rejected" }, t.endStatusRejected),
          React.createElement("option", { value: "cancelled" }, t.endStatusCancelled),
        ),
      ),
  );
}
