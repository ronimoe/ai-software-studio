"use client";

import { Switch } from "@/components/ui/switch";
import { useDispatchStatus } from "@/features/dispatch/use-dispatch-status";
import { usePauseDispatch, useResumeDispatch } from "@/features/dispatch/use-toggle-autorun";
import { useDispatchEvents } from "@/features/dispatch/use-dispatch-events";

export function DispatchControl() {
  useDispatchEvents();
  const { data: status } = useDispatchStatus();
  const pause = usePauseDispatch();
  const resume = useResumeDispatch();

  const running = status?.running ?? false;
  const pending = pause.isPending || resume.isPending;

  function onToggle(next: boolean) {
    if (next) resume.mutate();
    else pause.mutate();
  }

  let line: string;
  if (!status) {
    line = "…";
  } else if (!status.running) {
    line = "Paused";
  } else if (status.queued === 0 && status.currentTask === null) {
    line = "Idle";
  } else {
    line = `${status.queued} queued`;
    if (status.currentTask) line += ` · #${status.currentTask.replace("task-", "")}`;
  }

  return (
    <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
      <Switch
        aria-label="Auto-run"
        checked={running}
        onCheckedChange={onToggle}
        disabled={pending || !status}
      />
      <span className="tabular-nums">{line}</span>
    </div>
  );
}
