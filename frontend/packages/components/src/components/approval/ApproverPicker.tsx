import { useMemo } from "react";
import { SearchSelect } from "../ui/search-select";

export interface ApproverRef {
  kind: "role" | "engineer";
  id: string;
  label: string;
}

export interface ApproverOption {
  id: string | number;
  name: string;
}

export interface ApproverPickerLabels {
  roleTab: string;
  engineerTab: string;
  selectPlaceholder: string;
  searchPlaceholder: string;
  emptyText: string;
}

export interface ApproverPickerProps {
  value?: ApproverRef;
  onChange: (ref: ApproverRef) => void;
  roles: ApproverOption[];
  engineers: ApproverOption[];
  labels: ApproverPickerLabels;
}

export function ApproverPicker({
  value,
  onChange,
  roles,
  engineers,
  labels,
}: ApproverPickerProps) {
  const kind = value?.kind ?? "role";

  const options = useMemo(() => {
    if (kind === "role") {
      return roles.map((r) => ({ value: `role:${r.id}`, label: r.name }));
    }
    return engineers.map((e) => ({ value: `engineer:${e.id}`, label: e.name }));
  }, [kind, roles, engineers]);

  const selectedValue = value ? `${value.kind}:${value.id}` : "";

  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <button
          type="button"
          className={kind === "role" ? "bg-primary text-primary-foreground" : "bg-muted"}
          onClick={() => onChange({ kind: "role", id: "", label: "" })}
        >
          {labels.roleTab}
        </button>
        <button
          type="button"
          className={kind === "engineer" ? "bg-primary text-primary-foreground" : "bg-muted"}
          onClick={() => onChange({ kind: "engineer", id: "", label: "" })}
        >
          {labels.engineerTab}
        </button>
      </div>
      <SearchSelect
        value={selectedValue}
        onValueChange={(v) => {
          const opt = options.find((o) => o.value === v);
          if (!opt) return;
          const [k, id] = v.split(":") as [ApproverRef["kind"], string];
          onChange({ kind: k, id, label: opt.label });
        }}
        options={options}
        placeholder={labels.selectPlaceholder}
        searchPlaceholder={labels.searchPlaceholder}
        emptyText={labels.emptyText}
      />
    </div>
  );
}
