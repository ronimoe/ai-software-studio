"use client";

import { Loader2, GitBranch, Play, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useCreateWorktree } from "@/features/worktrees/use-create-worktree";
import { useStartTask } from "@/features/runs/use-start-task";
import { useStopTask } from "@/features/runs/use-stop-task";
import type { TaskStatus } from "@/lib/bindings";

interface Props {
  taskId: string;
  status: TaskStatus;
}

export function StartButton({ taskId, status }: Props) {
  const createWorktree = useCreateWorktree();
  const startTask = useStartTask();
  const stopTask = useStopTask();

  if (status === "draft") {
    return (
      <Button size="sm" onClick={() => createWorktree.mutate(taskId)} disabled={createWorktree.isPending}>
        {createWorktree.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <GitBranch className="mr-1 h-3 w-3" />}
        Create worktree
      </Button>
    );
  }
  if (status === "worktreeCreated" || status === "stopped" || status === "failed") {
    return (
      <Button size="sm" onClick={() => startTask.mutate(taskId)} disabled={startTask.isPending}>
        {startTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Play className="mr-1 h-3 w-3" />}
        Start
      </Button>
    );
  }
  if (status === "running" || status === "verificationRunning") {
    return (
      <Button size="sm" variant="destructive" onClick={() => stopTask.mutate(taskId)} disabled={stopTask.isPending}>
        {stopTask.isPending ? <Loader2 className="mr-1 h-3 w-3 animate-spin" /> : <Square className="mr-1 h-3 w-3" />}
        Stop
      </Button>
    );
  }
  return null;
}
