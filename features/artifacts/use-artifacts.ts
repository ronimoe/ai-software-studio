import { useQuery } from "@tanstack/react-query";

// TODO(phase-2): replace with `tauri.listArtifacts(taskId)` once the
// Rust artifact store lands (see src-tauri/src/artifacts/mod.rs).
export type ArtifactKind = "log" | "diff" | "screenshot" | "report";

export interface Artifact {
  id: string;
  taskId: string;
  kind: ArtifactKind;
  path: string;
  createdAt: string;
}

export function useArtifacts(taskId: string | null) {
  return useQuery({
    queryKey: ["artifacts", taskId],
    enabled: taskId !== null,
    queryFn: async (): Promise<Artifact[]> => [],
  });
}
