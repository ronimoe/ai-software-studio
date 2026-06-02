"use client";

import { X, Clock } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { useDequeueTask } from "@/features/dispatch/use-dequeue-task";
import type { Task } from "@/lib/tauri";

const statusColor: Record<string, string> = {
  draft: "bg-muted text-muted-foreground",
  queued: "bg-primary/15 text-primary",
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
  const dequeue = useDequeueTask();
  const isQueued = task.status === "queued";

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "w-full cursor-pointer rounded-lg border border-border/60 bg-card/60 p-3 text-left transition-colors",
        "hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active && "border-primary/60 bg-primary/10",
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          #{task.id.replace("task-", "")}
        </span>
        <Badge className={cn("gap-1 text-[10px]", statusColor[task.status] ?? "bg-muted")}>
          {isQueued && <Clock className="h-2.5 w-2.5" />}
          {task.status}
          {isQueued && (
            <button
              type="button"
              aria-label="Remove from queue"
              className="ml-0.5 rounded-sm hover:bg-primary/20 focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring disabled:opacity-50"
              disabled={dequeue.isPending}
              onClick={(e) => {
                e.stopPropagation();
                dequeue.mutate(task.id);
              }}
              onKeyDown={(e) => e.stopPropagation()}
            >
              <X className="h-2.5 w-2.5" />
            </button>
          )}
        </Badge>
      </div>
      <p className="mt-1.5 line-clamp-2 text-sm leading-snug">{task.title}</p>
      {task.selectedEngine && (
        <p className="mt-2 text-[11px] text-muted-foreground">via {task.selectedEngine}</p>
      )}
    </div>
  );
}
