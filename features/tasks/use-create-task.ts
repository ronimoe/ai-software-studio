import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";
import type { CreateTaskRequest } from "@/lib/bindings";

export function useCreateTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (request: CreateTaskRequest) => {
      const result = await tauri.createTask(request);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (task) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", task.projectId] });
    },
  });
}
