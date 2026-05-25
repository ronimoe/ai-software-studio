import { useEffect, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { events } from "@/lib/bindings";

export interface TerminalLine {
  text: string;
  stream: "stdout" | "stderr";
  ts: number;
}

export function useTaskOutput(taskId: string | null) {
  const [lines, setLines] = useState<TerminalLine[]>([]);
  const [exitCode, setExitCode] = useState<number | null | undefined>(undefined);

  useEffect(() => {
    if (!taskId) return;
    let unlistenOut: UnlistenFn | undefined;
    let unlistenExit: UnlistenFn | undefined;

    (async () => {
      try {
        unlistenOut = await events.taskOutput.listen((e) => {
          if (e.payload.taskId !== taskId) return;
          setLines((prev) => [
            ...prev,
            { text: e.payload.text, stream: e.payload.stream, ts: Date.now() },
          ]);
        });
        unlistenExit = await events.taskExit.listen((e) => {
          if (e.payload.taskId !== taskId) return;
          setExitCode(e.payload.exitCode);
        });
      } catch {
        // Dev mode without Tauri runtime — events won't fire, fine.
      }
    })();

    return () => {
      unlistenOut?.();
      unlistenExit?.();
    };
  }, [taskId]);

  return { lines, exitCode };
}
