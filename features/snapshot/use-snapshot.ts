import { useQuery } from "@tanstack/react-query";
import { mockSnapshot } from "@/lib/mock-data";
import type { SnapshotMetric } from "@/lib/types";

// TODO(phase-2): replace queryFn with `tauri.getEngineeringSnapshot()`
// once aggregate metrics land in Rust.
export function useSnapshot() {
  return useQuery({
    queryKey: ["snapshot"],
    queryFn: async (): Promise<SnapshotMetric[]> => mockSnapshot,
  });
}
