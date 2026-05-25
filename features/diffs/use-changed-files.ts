import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useChangedFiles(taskId: string | null) {
  return useQuery({
    queryKey: ["changed-files", taskId],
    queryFn: async () => {
      const result = await tauri.getChangedFiles(taskId!);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!taskId,
    refetchInterval: 5_000,
  });
}
