import { useMutation } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useRenderPrReport() {
  return useMutation({
    mutationFn: async (taskId: string) => {
      const result = await tauri.renderPrReport(taskId);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });
}
