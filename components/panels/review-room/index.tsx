"use client";

import { PanelFrame } from "@/components/layout/panel-frame";
import { useUiStore } from "@/stores/ui-store";
import { useVerification } from "@/features/verification/use-verification";
import { useTask } from "@/features/tasks/use-tasks";
import { StatusPill } from "./status-pill";
import { EvidenceArtifacts } from "./evidence-artifacts";
import { RunVerificationButton } from "./run-verification-button";

export function ReviewRoomPanel() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data: task } = useTask(activeTaskId);
  const { data: runs = [] } = useVerification(activeTaskId);
  const latest = runs[0];
  const hasWorktree = !!task?.worktreePath;

  return (
    <PanelFrame
      title="Review Room"
      badge="Verification"
      actions={
        activeTaskId ? (
          <RunVerificationButton taskId={activeTaskId} hasWorktree={hasWorktree} />
        ) : undefined
      }
    >
      <div className="space-y-3">
        {!activeTaskId && (
          <p className="text-xs text-muted-foreground">Select a task to see verification.</p>
        )}
        {activeTaskId && !latest && (
          <p className="text-xs text-muted-foreground">No verification run yet.</p>
        )}
        {latest && (
          <div className="grid grid-cols-2 gap-2 md:grid-cols-5">
            {latest.checks.map((c) => (
              <StatusPill key={c.kind} kind={c.kind} status={c.status} />
            ))}
          </div>
        )}
        <EvidenceArtifacts run={latest} />
      </div>
    </PanelFrame>
  );
}
