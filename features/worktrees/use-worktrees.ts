import { useQuery } from "@tanstack/react-query";

// TODO(phase-2): replace with `tauri.listWorktrees(taskId)` once the
// Rust git worktree service lands (see src-tauri/src/git/mod.rs).
export interface Worktree {
  taskId: string;
  branch: string;
  path: string;
}

export function useWorktrees(taskId: string | null) {
  return useQuery({
    queryKey: ["worktrees", taskId],
    enabled: taskId !== null,
    queryFn: async (): Promise<Worktree[]> => [],
  });
}
