import { useQuery } from "@tanstack/react-query";
import { mockConversation } from "@/lib/mock-data";
import type { ConversationMessage } from "@/lib/types";

// TODO(phase-2): replace queryFn with `tauri.listConversation(taskId)`
// once the conversation log service lands.
export function useConversation(taskId: string | null) {
  return useQuery({
    queryKey: ["conversation", taskId],
    enabled: taskId !== null,
    queryFn: async (): Promise<ConversationMessage[]> => mockConversation,
  });
}
