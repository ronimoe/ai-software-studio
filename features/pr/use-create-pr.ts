import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";
import type { CreatePrRequest } from "@/lib/bindings";

export function useCreatePr() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (request: CreatePrRequest) => {
      const result = await tauri.createPr(request);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (_pr, request) => {
      queryClient.invalidateQueries({ queryKey: ["task", request.taskId] });
    },
  });
}
