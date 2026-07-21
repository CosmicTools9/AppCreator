import { useEffect, useState } from "react";
import { useT } from "../locales";

type Status = "checking..." | "running" | "error" | "unreachable";

export function StatusBadge() {
  const { t } = useT();
  const [status, setStatus] = useState<Status>("checking...");

  useEffect(() => {
    let cancelled = false;
    fetch("/api/creator/status")
      .then((r) => r.json())
      .then((data) => {
        if (!cancelled) setStatus(data.status === "running" ? "running" : "error");
      })
      .catch(() => {
        if (!cancelled) setStatus("unreachable");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const label =
    status === "running"
      ? t("status.online")
      : status === "unreachable"
        ? t("status.offline")
        : status === "error"
          ? t("status.error")
          : t("status.checking");

  return (
    <span className="status-badge" data-status={status}>
      {label}
    </span>
  );
}
