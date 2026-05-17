import { useQuery } from "@tanstack/react-query";
import { mockGraphEdges, mockGraphNodes } from "@/lib/mock-data";
import type { ContextGraphEdge, ContextGraphNode } from "@/lib/types";

export interface ContextGraph {
  nodes: ContextGraphNode[];
  edges: ContextGraphEdge[];
}

// TODO(phase-2): replace queryFn with `tauri.getContextGraph(taskId)`
// once the real dependency-graph builder lands.
export function useContextGraph(taskId: string | null) {
  return useQuery({
    queryKey: ["context-graph", taskId],
    enabled: taskId !== null,
    queryFn: async (): Promise<ContextGraph> => ({
      nodes: mockGraphNodes,
      edges: mockGraphEdges,
    }),
  });
}
