"use client";

import { useState, type ReactNode } from "react";
import { FileEdit, FilePlus, FileMinus, FileQuestion, GitMerge } from "lucide-react";
import type { ChangeStatus, ChangedFile } from "@/lib/bindings";
import { useChangedFiles } from "@/features/diffs/use-changed-files";
import { DiffViewerDialog } from "./diff-viewer-dialog";

interface Props {
  taskId: string;
}

const ICON: Record<ChangeStatus, ReactNode> = {
  added: <FilePlus className="h-3 w-3 text-emerald-500" />,
  modified: <FileEdit className="h-3 w-3 text-amber-500" />,
  deleted: <FileMinus className="h-3 w-3 text-red-500" />,
  renamed: <FileEdit className="h-3 w-3 text-blue-500" />,
  untracked: <FileQuestion className="h-3 w-3 text-muted-foreground" />,
  conflicted: <GitMerge className="h-3 w-3 text-red-500" />,
};

export function ChangedFilesPanel({ taskId }: Props) {
  const { data: files = [], isLoading } = useChangedFiles(taskId);
  const [openPath, setOpenPath] = useState<string | null>(null);

  return (
    <div className="space-y-1">
      {isLoading && <p className="text-xs text-muted-foreground">Loading changes…</p>}
      {!isLoading && files.length === 0 && (
        <p className="text-xs text-muted-foreground">No changes yet.</p>
      )}
      {files.map((f: ChangedFile) => (
        <button
          key={f.path}
          onClick={() => setOpenPath(f.path)}
          className="flex w-full items-center gap-2 rounded px-1.5 py-1 text-left text-xs hover:bg-muted/60"
        >
          {ICON[f.status]}
          <span className="flex-1 truncate font-mono">{f.path}</span>
          <span className="text-[10px] text-muted-foreground">
            +{f.additions} −{f.deletions}
          </span>
        </button>
      ))}
      <DiffViewerDialog
        taskId={taskId}
        path={openPath}
        onClose={() => setOpenPath(null)}
      />
    </div>
  );
}
