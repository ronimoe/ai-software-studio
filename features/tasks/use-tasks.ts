import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useTasks(projectId: string) {
  return useQuery({
    queryKey: ["tasks", projectId],
    queryFn: async () => {
      const result = await tauri.listTasks(projectId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!projectId,
  });
}

export function useTask(taskId: string | null) {
  return useQuery({
    queryKey: ["task", taskId],
    queryFn: async () => {
      const result = await tauri.getTask(taskId!);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!taskId,
  });
}
