"use client";

import { Loader2, PlayCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useRunVerification } from "@/features/verification/use-run-verification";

interface Props {
  taskId: string;
  hasWorktree: boolean;
}

export function RunVerificationButton({ taskId, hasWorktree }: Props) {
  const mutation = useRunVerification();
  return (
    <Button
      size="sm"
      variant="outline"
      onClick={() => mutation.mutate(taskId)}
      disabled={!hasWorktree || mutation.isPending}
    >
      {mutation.isPending ? (
        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
      ) : (
        <PlayCircle className="mr-1 h-3 w-3" />
      )}
      Run verification
    </Button>
  );
}
