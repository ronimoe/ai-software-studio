import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useFileDiff(taskId: string | null, path: string | null) {
  return useQuery({
    queryKey: ["file-diff", taskId, path],
    queryFn: async () => {
      const result = await tauri.getFileDiff(taskId!, path!);
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    enabled: !!taskId && !!path,
  });
}
