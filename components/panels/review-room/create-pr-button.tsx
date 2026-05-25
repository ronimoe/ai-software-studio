"use client";

import { useState } from "react";
import { Loader2, GitPullRequest, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useDetectGithub } from "@/features/pr/use-detect-github";
import { useCreatePr } from "@/features/pr/use-create-pr";

interface Props {
  taskId: string;
  hasWorktree: boolean;
}

export function CreatePrButton({ taskId, hasWorktree }: Props) {
  const { data: gh } = useDetectGithub();
  const create = useCreatePr();
  const [prUrl, setPrUrl] = useState<string | null>(null);

  const disabled =
    !hasWorktree ||
    gh?.auth !== "authed" ||
    create.isPending;

  const tooltip =
    !hasWorktree
      ? "Create a worktree first"
      : gh?.auth === "notInstalled"
        ? "`gh` not installed — `brew install gh`"
        : gh?.auth === "notAuthed"
          ? "Run `gh auth login` first"
          : "Push branch and open a PR";

  const onClick = async () => {
    const pr = await create.mutateAsync({ taskId, baseBranch: null, draft: false });
    setPrUrl(pr.url);
  };

  if (prUrl) {
    return (
      <a href={prUrl} target="_blank" rel="noreferrer">
        <Button size="sm" variant="default">
          <ExternalLink className="mr-1 h-3 w-3" />
          Open PR
        </Button>
      </a>
    );
  }

  return (
    <Button size="sm" onClick={onClick} disabled={disabled} title={tooltip}>
      {create.isPending ? (
        <Loader2 className="mr-1 h-3 w-3 animate-spin" />
      ) : (
        <GitPullRequest className="mr-1 h-3 w-3" />
      )}
      Create PR
    </Button>
  );
}
