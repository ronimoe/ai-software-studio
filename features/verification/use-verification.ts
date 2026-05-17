import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useVerification(taskId: string | null) {
  return useQuery({
    queryKey: ["verification", taskId],
    queryFn: async () => {
      const result = await tauri.listVerification(taskId!);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!taskId,
  });
}
