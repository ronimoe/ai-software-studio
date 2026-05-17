"use client";

import { Check, CircleAlert, Clock, Minus, X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { VerificationRun } from "@/lib/tauri";

type Status = VerificationRun["checks"][number]["status"];

const meta: Record<Status, { icon: typeof Check; className: string }> = {
  passed:    { icon: Check,       className: "bg-success/15 text-success" },
  failed:    { icon: X,           className: "bg-destructive/15 text-destructive" },
  warning:   { icon: CircleAlert, className: "bg-warning/15 text-warning" },
  running:   { icon: Clock,       className: "bg-primary/15 text-primary" },
  notRun:    { icon: Minus,       className: "bg-muted text-muted-foreground" },
  skipped:   { icon: Minus,       className: "bg-muted text-muted-foreground" },
};

export function StatusPill({ kind, status }: { kind: string; status: Status }) {
  const { icon: Icon, className } = meta[status];
  return (
    <div className={cn("flex items-center justify-between gap-2 rounded-md border border-border/40 px-3 py-2", className)}>
      <span className="text-xs font-medium uppercase tracking-wider">{kind}</span>
      <Icon className="h-4 w-4" />
    </div>
  );
}
