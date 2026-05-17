import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function useProjects() {
  return useQuery({
    queryKey: ["projects"],
    queryFn: async () => {
      const result = await tauri.listProjects();
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
  });
}
