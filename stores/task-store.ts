import { create } from "zustand";

interface LogLine {
  id: string;
  taskId: string;
  body: string;
  timestamp: string;
  kind: "stdout" | "stderr" | "system";
}

interface TaskState {
  streamingLog: Record<string, LogLine[]>;
  appendLog: (taskId: string, line: Omit<LogLine, "taskId">) => void;
  clearLog: (taskId: string) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  streamingLog: {
    "task-042": [
      { id: "l1", taskId: "task-042", body: "Reading src/auth/jwt.ts", timestamp: "10:14:02", kind: "stdout" },
      { id: "l2", taskId: "task-042", body: "Creating src/auth/magic-link.ts", timestamp: "10:14:11", kind: "stdout" },
      { id: "l3", taskId: "task-042", body: "Running pnpm test...", timestamp: "10:18:00", kind: "system" },
      { id: "l4", taskId: "task-042", body: "142 passed, 0 failed", timestamp: "10:18:18", kind: "stdout" },
      { id: "l5", taskId: "task-042", body: "build: type error in middleware.ts:88", timestamp: "10:21:04", kind: "stderr" },
    ],
  },
  appendLog: (taskId, line) =>
    set((s) => ({
      streamingLog: {
        ...s.streamingLog,
        [taskId]: [...(s.streamingLog[taskId] ?? []), { ...line, taskId }],
      },
    })),
  clearLog: (taskId) =>
    set((s) => ({ streamingLog: { ...s.streamingLog, [taskId]: [] } })),
}));
