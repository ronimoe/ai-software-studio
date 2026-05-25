"use client";

import { useMemo } from "react";
import { parseDiff, Diff, Hunk } from "react-diff-view";
import "react-diff-view/style/index.css";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useFileDiff } from "@/features/diffs/use-file-diff";

interface Props {
  taskId: string;
  path: string | null;
  onClose: () => void;
}

export function DiffViewerDialog({ taskId, path, onClose }: Props) {
  const { data: diffText = "", isLoading } = useFileDiff(taskId, path);

  const files = useMemo(() => {
    if (!diffText) return [];
    try {
      return parseDiff(diffText);
    } catch {
      return [];
    }
  }, [diffText]);

  return (
    <Dialog open={!!path} onOpenChange={(o) => { if (!o) onClose(); }}>
      <DialogContent className="max-w-4xl">
        <DialogHeader>
          <DialogTitle className="font-mono text-sm">{path}</DialogTitle>
        </DialogHeader>
        <div className="max-h-[60vh] overflow-auto">
          {isLoading && <p className="text-xs text-muted-foreground">Loading diff…</p>}
          {!isLoading && files.length === 0 && (
            <p className="text-xs text-muted-foreground">No diff to show.</p>
          )}
          {files.map((file, i: number) => (
            <Diff key={i} viewType="split" diffType={file.type} hunks={file.hunks ?? []}>
              {(hunks) => hunks.map((h) => <Hunk key={h.content} hunk={h} />)}
            </Diff>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
