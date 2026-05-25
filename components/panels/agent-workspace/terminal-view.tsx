"use client";

import { useEffect, useRef } from "react";
import { useTaskOutput } from "@/features/runs/use-task-output";

interface Props {
  taskId: string;
}

export function TerminalView({ taskId }: Props) {
  const { lines, exitCode } = useTaskOutput(taskId);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    ref.current?.scrollTo({ top: ref.current.scrollHeight });
  }, [lines.length]);

  return (
    <div className="rounded-md border bg-muted/30 font-mono">
      <div
        ref={ref}
        className="max-h-[280px] min-h-[120px] overflow-auto whitespace-pre-wrap break-words p-3 text-[11px] leading-snug"
      >
        {lines.length === 0 && (
          <span className="text-muted-foreground">Waiting for output…</span>
        )}
        {lines.map((l, i) => (
          <div
            key={i}
            className={l.stream === "stderr" ? "text-amber-500" : ""}
          >
            {l.text}
          </div>
        ))}
        {exitCode !== undefined && (
          <div className="mt-2 italic text-muted-foreground">
            — process exited{exitCode === null ? " (signaled)" : ` (code ${exitCode})`} —
          </div>
        )}
      </div>
    </div>
  );
}
