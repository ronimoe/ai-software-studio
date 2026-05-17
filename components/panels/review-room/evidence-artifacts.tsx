"use client";

import { cn } from "@/lib/utils";
import type { VerificationRun } from "@/lib/bindings";

const statusColor: Record<string, string> = {
  passed: "bg-success/70",
  warning: "bg-warning/70",
  failed: "bg-destructive/70",
  skipped: "bg-muted/70",
};

interface Props {
  run?: VerificationRun;
}

export function EvidenceArtifacts({ run }: Props) {
  const checks = run?.checks ?? [];
  const maxMs = Math.max(1, ...checks.map((c) => c.durationMs ?? 0));

  return (
    <div className="rounded-md border border-border/40 bg-muted/30 p-3">
      <h4 className="mb-2 text-[10px] font-medium uppercase tracking-wider text-muted-foreground">
        Evidence Artifacts
      </h4>
      {checks.length === 0 ? (
        <p className="text-[11px] text-muted-foreground">No checks recorded.</p>
      ) : (
        <div className="flex h-12 items-end gap-1.5">
          {checks.map((c) => (
            <div key={c.kind} className="flex flex-1 flex-col items-center gap-1">
              <div
                title={`${c.kind}: ${c.status}${c.durationMs != null ? ` · ${c.durationMs}ms` : ""}`}
                style={{ height: `${Math.max(8, ((c.durationMs ?? 0) / maxMs) * 100)}%` }}
                className={cn("w-full rounded-sm", statusColor[c.status] ?? "bg-primary/60")}
              />
              <span className="text-[9px] uppercase tracking-wider text-muted-foreground">
                {c.kind}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
