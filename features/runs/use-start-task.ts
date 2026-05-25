import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useStartTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (taskId: string) => {
      const result = await tauri.startTask(taskId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (task) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", task.projectId] });
      queryClient.invalidateQueries({ queryKey: ["task", task.id] });
    },
  });
}
