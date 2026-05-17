"use client";

import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import type { Task } from "@/lib/tauri";

const statusColor: Record<string, string> = {
  draft: "bg-muted text-muted-foreground",
  running: "bg-warning/20 text-warning",
  reviewReady: "bg-primary/20 text-primary",
  approved: "bg-success/20 text-success",
  changesRequested: "bg-destructive/20 text-destructive",
};

interface TaskCardProps {
  task: Task;
  active: boolean;
  onSelect: () => void;
}

export function TaskCard({ task, active, onSelect }: TaskCardProps) {
  return (
    <button
      onClick={onSelect}
      className={cn(
        "w-full rounded-lg border border-border/60 bg-card/60 p-3 text-left transition-colors",
        "hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active && "border-primary/60 bg-primary/10",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          #{task.id.replace("task-", "")}
        </span>
        <Badge className={cn("text-[10px]", statusColor[task.status] ?? "bg-muted")}>{task.status}</Badge>
      </div>
      <p className="mt-1.5 line-clamp-2 text-sm leading-snug">{task.title}</p>
      {task.selectedEngine && (
        <p className="mt-2 text-[11px] text-muted-foreground">via {task.selectedEngine}</p>
      )}
    </button>
  );
}
