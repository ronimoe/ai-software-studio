"use client";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { PanelFrame } from "@/components/layout/panel-frame";
import { useUiStore } from "@/stores/ui-store";
import { useTask } from "@/features/tasks/use-tasks";
import { AcceptanceList } from "./acceptance-list";
import { ActivityLog } from "./activity-log";
import { StartButton } from "./start-button";

export function AgentWorkspacePanel() {
  const activeTaskId = useUiStore((s) => s.activeTaskId);
  const { data: task } = useTask(activeTaskId);

  if (!activeTaskId || !task) {
    return (
      <PanelFrame title="Agent Workspace" subtitle="Select a task from the Task Board to begin.">
        <p className="text-xs text-muted-foreground">No task selected.</p>
      </PanelFrame>
    );
  }

  return (
    <PanelFrame
      title="Agent Workspace"
      subtitle="Watch first. Evidence-first. Human accountable."
      badge={task.status}
      actions={
        task.status === "draft" ? (
          <StartButton taskId={task.id} />
        ) : (
          <div className="flex items-center gap-1">
            <Button size="sm" variant="ghost">Request Changes</Button>
            <Button size="sm">Approve</Button>
          </div>
        )
      }
    >
      <div className="space-y-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-muted-foreground">Task #{task.id.replace("task-", "")}</span>
            <Badge variant="outline" className="text-[10px]">{task.risk}</Badge>
          </div>
          <h3 className="mt-1 text-base font-semibold">{task.title}</h3>
          <p className="mt-1 text-sm text-muted-foreground">{task.description}</p>
          {task.worktreePath && (
            <p className="mt-1 font-mono text-[11px] text-muted-foreground">
              worktree: {task.worktreePath}
            </p>
          )}
        </div>

        <div>
          <h4 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Acceptance Criteria
          </h4>
          <AcceptanceList items={task.acceptanceCriteria} />
        </div>

        <div>
          <h4 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Activity
          </h4>
          <ActivityLog taskId={task.id} />
        </div>
      </div>
    </PanelFrame>
  );
}
