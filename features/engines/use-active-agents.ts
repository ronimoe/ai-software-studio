import { useQuery } from "@tanstack/react-query";
import { mockActiveAgents } from "@/lib/mock-data";
import type { ActiveAgent } from "@/lib/types";

// TODO(phase-2): replace queryFn with `tauri.listActiveAgents()` once
// the engine runner reports live status.
export function useActiveAgents() {
  return useQuery({
    queryKey: ["active-agents"],
    queryFn: async (): Promise<ActiveAgent[]> => mockActiveAgents,
  });
}
