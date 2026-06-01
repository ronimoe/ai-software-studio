"use client";

import type { ReactNode } from "react";
import { Loader2, GitBranch, Play, Square, ListPlus, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useCreateWorktree } from "@/features/worktrees/use-create-worktree";
import { useStartTask } from "@/features/runs/use-start-task";
import { useStopTask } from "@/features/runs/use-stop-task";
import { useEnqueueTask } from "@/features/dispatch/use-enqueue-task";
import type { TaskStatus } from "@/lib/bindings";

interface Props {
  taskId: string;
  status: TaskStatus;
  selectedEngine: string | null;
}

const ENQUEUE_ELIGIBLE: TaskStatus[] = ["draft", "stopped", "failed", "changesRequested"];

export function StartButton({ taskId, status, selectedEngine }: Props) {
  const createWorktree = useCreateWorktree();
  const startTask = useStartTask();
  const stopTask = useStopTask();
  const enqueueTask = useEnqueueTask();

  if (status === "queued") {
    return (
      <Badge variant="outline" className="gap-1 text-[11px] text-muted-foreground">
        <Clock className="h-3 w-3" /> Queued
      </Badge>
    );
  }

  const engineEligible = selectedEngine === null || selectedEngine === "claude-code";
  const statusEligible = ENQUEUE_ELIGIBLE.includes(status);

  const queueButton = statusEligible ? (
    <span title={engineEligible ? undefined : "Only claude-code is dispatchable"}>
      <Button
        size="sm"
        variant="outline"
        onClick={() => enqueueTask.mutate(taskId)}
        disabled={!engineEligible || enqueueTask.isPending}
      >
        {enqueueTask.isPending ? (
          <Loader2 className="mr-1 h-3 w-3 animate-spin" />
        ) : (
          <ListPlus className="mr-1 h-3 w-3" />
        )}
        Queue
      </Button>
    </span>
  ) : null;

  let primary: ReactNode = null;
  if (status === "draft") {
    primary = (
      <Button size="sm" onClick={() => createWorktree.mutate(taskId)} disabled={createWorktree.isPending}>
        {createWorktree.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <GitBranch className="mr-1 h-3 w-3" />}
        Create worktree
      </Button>
    );
  } else if (status === "worktreeCreated" || status === "stopped" || status === "failed") {
    primary = (
      <Button size="sm" onClick={() => startTask.mutate(taskId)} disabled={startTask.isPending}>
        {startTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Play className="mr-1 h-3 w-3" />}
        Start
      </Button>
    );
  } else if (status === "running" || status === "verificationRunning") {
    primary = (
      <Button size="sm" variant="destructive" onClick={() => stopTask.mutate(taskId)} disabled={stopTask.isPending}>
        {stopTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Square className="mr-1 h-3 w-3" />}
        Stop
      </Button>
    );
  }

  if (!primary && !queueButton) return null;

  return (
    <div className="flex items-center gap-2">
      {primary}
      {queueButton}
    </div>
  );
}
