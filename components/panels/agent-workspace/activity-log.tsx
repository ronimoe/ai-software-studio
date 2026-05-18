"use client";

import { cn } from "@/lib/utils";
import { useTaskStore } from "@/stores/task-store";

const kindColor = {
  stdout: "text-foreground",
  stderr: "text-destructive",
  system: "text-primary",
};

const EMPTY_LINES: never[] = [];

export function ActivityLog({ taskId }: { taskId: string }) {
  const lines = useTaskStore((s) => s.streamingLog[taskId] ?? EMPTY_LINES);
  if (lines.length === 0) {
    return <p className="text-xs text-muted-foreground">No activity yet.</p>;
  }
  return (
    <div className="rounded-md border border-border/60 bg-muted/30 p-3 font-mono text-[11px] leading-relaxed">
      {lines.map((l) => (
        <div key={l.id} className="flex gap-2">
          <span className="text-muted-foreground">{l.timestamp}</span>
          <span className={cn(kindColor[l.kind])}>{l.body}</span>
        </div>
      ))}
    </div>
  );
}
