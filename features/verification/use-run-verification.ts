import { useMutation, useQueryClient } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useRunVerification() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (taskId: string) => {
      const result = await tauri.runVerification(taskId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: (run) => {
      queryClient.invalidateQueries({ queryKey: ["verification", run.taskId] });
    },
  });
}
