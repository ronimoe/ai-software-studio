"use client";

import { Loader2, GitBranch } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useCreateWorktree } from "@/features/worktrees/use-create-worktree";

interface Props {
  taskId: string;
}

export function StartButton({ taskId }: Props) {
  const mutation = useCreateWorktree();
  return (
    <Button
      size="sm"
      onClick={() => mutation.mutate(taskId)}
      disabled={mutation.isPending}
    >
      {mutation.isPending ? (
        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
      ) : (
        <GitBranch className="mr-1 h-3 w-3" />
      )}
      Create worktree
    </Button>
  );
}
