"use client";

import { PanelFrame } from "@/components/layout/panel-frame";
import { useUiStore } from "@/stores/ui-store";
import { useVerification } from "@/features/verification/use-verification";
import { StatusPill } from "./status-pill";
import { EvidenceArtifacts } from "./evidence-artifacts";

export function ReviewRoomPanel() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data: runs = [] } = useVerification(activeTaskId);
  const latest = runs[0];

  return (
    <PanelFrame title="Review Room" badge="Verification">
      <div className="space-y-3">
        {!latest && (
          <p className="text-xs text-muted-foreground">No verification run yet.</p>
        )}
        {latest && (
          <div className="grid grid-cols-2 gap-2 md:grid-cols-5">
            {latest.checks.map((c) => (
              <StatusPill key={c.kind} kind={c.kind} status={c.status} />
            ))}
          </div>
        )}
        <EvidenceArtifacts />
      </div>
    </PanelFrame>
  );
}
